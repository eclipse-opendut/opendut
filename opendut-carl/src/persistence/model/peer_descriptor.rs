use std::ops::DerefMut;
use diesel::{Connection, PgConnection};

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::topology::Topology;
use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

use crate::persistence::error::{PersistenceError, PersistenceOperation, PersistenceResult};
use crate::persistence::model::persistable::network_interface_descriptor::{PersistableNetworkInterfaceDescriptor, PersistableNetworkInterfaceKindCan};
use crate::persistence::model::persistable::network_interface_kind::PersistableNetworkInterfaceKind;
use crate::persistence::model::persistable::peer_descriptor::PersistablePeerDescriptor;
use crate::persistence::Storage;
use super::{Persistable, persistable};

impl Persistable for PeerDescriptor {
    fn insert(self, _peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().deref_mut().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(_peer_id: PeerId, _storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        todo!("implement removal of peer_descriptors from database")
    }

    fn get(peer_id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        get_from_database(peer_id, storage.db.lock().unwrap().deref_mut())
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        list_database(storage.db.lock().unwrap().deref_mut())
    }
}

pub(super) fn insert_into_database(peer_descriptor: PeerDescriptor, connection: &mut PgConnection) -> PersistenceResult<()> {
    let PeerDescriptor { id: peer_id, name, location, network, topology, executors: _ } = peer_descriptor; //TODO persist executors
    let PeerNetworkDescriptor { interfaces, bridge_name } = network;

    PersistablePeerDescriptor {
        peer_id: peer_id.uuid,
        name: name.value(),
        location: location.map(|location| location.value()),
        network_bridge_name: bridge_name.map(|name| name.name()),
    }.insert(peer_id, connection)?;

    for interface in interfaces {
        persistable::network_interface_descriptor::insert_into_database(&interface, peer_id, connection)?;
    }

    let Topology { devices } = topology;

    for device in devices {
        crate::persistence::model::device_descriptor::insert_into_database(device, connection)?;
    }

    // TODO persist executors

    Ok(())
}

fn get_from_database(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Option<PeerDescriptor>> {
    let persistable_peer_descriptor = PersistablePeerDescriptor::get(peer_id, connection)?;
    persistable_peer_descriptor.map(|persistable_peer_descriptor| {

        let result = load_other_peer_descriptor_tables(
            persistable_peer_descriptor,
            PersistenceOperation::Get,
            connection,
        ).map_err(|cause| PersistenceError::get::<PeerDescriptor>(peer_id.uuid, cause).context("Failed to convert from database values to PeerDescriptor."))?;

        Ok(result)
    }).transpose()
}

fn list_database(connection: &mut PgConnection) -> PersistenceResult<Vec<PeerDescriptor>> {
    let persistable_peer_descriptors = PersistablePeerDescriptor::list(connection)?;

    let result = persistable_peer_descriptors.into_iter().map(|persistable_peer_descriptor| {
        let result = load_other_peer_descriptor_tables(
            persistable_peer_descriptor,
            PersistenceOperation::List,
            connection,
        ).map_err(|cause| PersistenceError::list::<PeerDescriptor>(cause).context("Failed to convert from database values to PeerDescriptor."))?;

        Ok(result)
    }).collect::<PersistenceResult<Vec<_>>>()
    .map_err(|cause| PersistenceError::list::<PeerDescriptor>(cause).context("Failed to convert from list of PersistablePeerDescriptors."))?;

    Ok(result)
}

fn load_other_peer_descriptor_tables(
    persistable_peer_descriptor: PersistablePeerDescriptor,
    operation: PersistenceOperation,
    connection: &mut PgConnection,
) -> PersistenceResult<PeerDescriptor> {
    let PersistablePeerDescriptor { peer_id, name, location, network_bridge_name } = persistable_peer_descriptor;
    let peer_id = PeerId::from(peer_id);

    let error = |cause: Box<dyn std::error::Error + Send + Sync>| {
        PersistenceError::new::<PeerDescriptor>(Some(peer_id.uuid), operation, Some(cause))
    };

    let name = PeerName::try_from(name)
        .map_err(|cause| error(Box::new(cause)))?;
    let location = location.map(PeerLocation::try_from).transpose()
        .map_err(|cause| error(Box::new(cause)))?;

    let network_bridge_name = network_bridge_name.map(NetworkInterfaceName::try_from)
        .transpose()
        .map_err(|cause| error(Box::new(cause)))?;


    let network_interfaces = {
        let persistable_network_interface_descriptors = persistable::network_interface_descriptor::list_filtered_by_peer_id(peer_id, connection)?;

        persistable_network_interface_descriptors.into_iter()
            .map(|(persistable_network_interface_descriptor, persistable_network_interface_kind_can)| {
                let PersistableNetworkInterfaceDescriptor { network_interface_id, name, kind, peer_id: _ } = persistable_network_interface_descriptor;

                let id = NetworkInterfaceId::from(network_interface_id);
                let name = NetworkInterfaceName::try_from(name)
                    .map_err(|cause| error(Box::new(cause)))?;

                let configuration = match kind {
                    PersistableNetworkInterfaceKind::Ethernet => NetworkInterfaceConfiguration::Ethernet,
                    PersistableNetworkInterfaceKind::Can => {
                        let PersistableNetworkInterfaceKindCan { network_interface_id: _, bitrate, sample_point_times_1000, fd, data_bitrate, data_sample_point_times_1000 } = persistable_network_interface_kind_can
                            .ok_or(PersistenceError::new::<PeerDescriptor>(Some(peer_id.uuid), operation, Option::<PersistenceError>::None))?;

                        let bitrate = u32::try_from(bitrate)
                            .map_err(|cause| error(Box::new(cause)))?;

                        let sample_point = u32::try_from(sample_point_times_1000)
                            .map_err(|cause| error(Box::new(cause)))?;
                        let sample_point = CanSamplePoint::try_from(sample_point)
                            .map_err(|cause| error(Box::new(cause)))?;

                        let data_bitrate = u32::try_from(data_bitrate)
                            .map_err(|cause| error(Box::new(cause)))?;

                        let data_sample_point = u32::try_from(data_sample_point_times_1000)
                            .map_err(|cause| error(Box::new(cause)))?;
                        let data_sample_point = CanSamplePoint::try_from(data_sample_point)
                            .map_err(|cause| error(Box::new(cause)))?;

                        NetworkInterfaceConfiguration::Can {
                            bitrate,
                            sample_point,
                            fd,
                            data_bitrate,
                            data_sample_point,
                        }
                    }
                };

                Ok(NetworkInterfaceDescriptor { id, name, configuration })
            }).collect::<PersistenceResult<Vec<_>>>()?
    };

    let devices = crate::persistence::model::device_descriptor::list_database_filtered_by_peer_id(peer_id, connection)?;

    Ok(PeerDescriptor {
        id: peer_id,
        name,
        location,
        network: PeerNetworkDescriptor {
            interfaces: network_interfaces,
            bridge_name: network_bridge_name,
        },
        topology: Topology {
            devices,
        },
        executors: ExecutorDescriptors { executors: Default::default() }, //TODO
    })
}

#[cfg(test)]
pub(super) mod tests {
    use opendut_types::peer::PeerId;
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag, Topology};
    use opendut_types::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    use crate::persistence::database;
    use super::*;

    #[tokio::test]
    async fn should_persist_peer_descriptor() -> anyhow::Result<()> {
        let mut db = database::testing::spawn_and_connect().await?;

        let testee = peer_descriptor()?;
        let peer_id = testee.id;

        let result = get_from_database(peer_id, &mut db.connection)?;
        assert!(result.is_none());
        let result = list_database(&mut db.connection)?;
        assert!(result.is_empty());

        insert_into_database(testee.clone(), &mut db.connection)?;

        let result = get_from_database(peer_id, &mut db.connection)?;
        assert_eq!(result, Some(testee.clone()));
        let result = list_database(&mut db.connection)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result.first(), Some(&testee));

        Ok(())
    }

    pub fn peer_descriptor() -> anyhow::Result<PeerDescriptor> {
        let network_interface_id1 = NetworkInterfaceId::random();

        Ok(PeerDescriptor {
            id: PeerId::random(),
            name: PeerName::try_from("testee_name")?,
            location: Some(PeerLocation::try_from("testee_location")?),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: network_interface_id1,
                        name: NetworkInterfaceName::try_from("eth0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                    NetworkInterfaceDescriptor {
                        id: NetworkInterfaceId::random(),
                        name: NetworkInterfaceName::try_from("can0")?,
                        configuration: NetworkInterfaceConfiguration::Can {
                            bitrate: 11111,
                            sample_point: CanSamplePoint::try_from(0.222)?,
                            fd: true,
                            data_bitrate: 33333,
                            data_sample_point: CanSamplePoint::try_from(0.444)?,
                        },
                    },
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br0")?),
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: DeviceId::random(),
                        name: DeviceName::try_from("device1")?,
                        description: Some(DeviceDescription::try_from("device1-description")?),
                        interface: network_interface_id1,
                        tags: vec![
                            DeviceTag::try_from("tag1")?,
                            DeviceTag::try_from("tag2")?,
                        ],
                    }
                ],
            },
            executors: ExecutorDescriptors { executors: vec![] }, //TODO
        })
    }
}
