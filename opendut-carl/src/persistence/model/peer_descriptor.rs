use std::ops::DerefMut;

use diesel::{Connection, PgConnection};
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::ExecutorDescriptors;
use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

use crate::persistence::error::{PersistenceError, PersistenceOperation, PersistenceResult};
use crate::persistence::model::persistable::network_interface_descriptor::{PersistableNetworkInterfaceDescriptor, PersistableNetworkInterfaceKindCan};
use crate::persistence::model::persistable::network_interface_kind::PersistableNetworkInterfaceKind;
use crate::persistence::model::persistable::peer_descriptor::PersistablePeerDescriptor;
use crate::persistence::Storage;
use super::{Persistable, persistable};

impl Persistable for PeerDescriptor {
    fn insert(self, peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().deref_mut().transaction::<_, PersistenceError, _>(|connection| {
            insert(self, peer_id, connection)
        })
    }

    fn remove(_peer_id: PeerId, _storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        todo!("implement removal of peer_descriptors from database")
    }

    fn get(peer_id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        get(peer_id, storage.db.lock().unwrap().deref_mut())
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        list(storage.db.lock().unwrap().deref_mut())
    }
}

fn insert(peer_descriptor: PeerDescriptor, _peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<()> {
    let PeerDescriptor { id: peer_id, name, location, network, topology: _, executors: _ } = peer_descriptor; //TODO persist topology and executors
    let PeerNetworkDescriptor { interfaces, bridge_name } = network;

    PersistablePeerDescriptor {
        peer_id: peer_id.uuid,
        name: name.value(),
        location: location.map(|location| location.value()),
        network_bridge_name: bridge_name.map(|name| name.name()),
    }.insert(peer_id, connection)?;

    for interface in interfaces {
        let network_interface_id = interface.id.uuid;

        let (kind, network_interface_kind_can) = match interface.configuration {
            NetworkInterfaceConfiguration::Ethernet => {
                (PersistableNetworkInterfaceKind::Ethernet, None)
            }
            NetworkInterfaceConfiguration::Can { bitrate, sample_point, fd, data_bitrate, data_sample_point } => {
                let bitrate = i32::try_from(bitrate)
                    .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;
                let sample_point_times_1000 = i32::try_from(sample_point.sample_point_times_1000())
                    .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;
                let data_bitrate = i32::try_from(data_bitrate)
                    .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;
                let data_sample_point_times_1000 = i32::try_from(data_sample_point.sample_point_times_1000())
                    .map_err(|cause| PersistenceError::insert::<PersistableNetworkInterfaceKindCan>(network_interface_id, cause))?;

                let network_interface_kind_can = PersistableNetworkInterfaceKindCan {
                    network_interface_id,
                    bitrate,
                    sample_point_times_1000,
                    fd,
                    data_bitrate,
                    data_sample_point_times_1000,
                };
                (PersistableNetworkInterfaceKind::Can, Some(network_interface_kind_can))
            }
        };
        let network_interface_descriptor = PersistableNetworkInterfaceDescriptor {
            network_interface_id,
            name: interface.name.name(),
            kind,
            peer_id: peer_id.uuid,
        };

        persistable::network_interface_descriptor::insert(network_interface_descriptor, network_interface_kind_can, interface.id, connection)?;
    }

    // TODO persist other fields

    Ok(())
}

fn get(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Option<PeerDescriptor>> {
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

fn list(connection: &mut PgConnection) -> PersistenceResult<Vec<PeerDescriptor>> {
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

    let persistable_network_interface_descriptors = persistable::network_interface_descriptor::list_filtered_by_peer_id(peer_id, connection)?;

    let error = |cause: Box<dyn std::error::Error + Send + Sync>| {
        PersistenceError::new::<PeerDescriptor>(Some(peer_id.uuid), operation, Some(cause))
    };

    let name = PeerName::try_from(name)
        .map_err(|cause| error(Box::new(cause)))?;
    let location = location.map(PeerLocation::try_from).transpose()
        .map_err(|cause| error(Box::new(cause)))?;

    let network_interfaces = persistable_network_interface_descriptors.into_iter()
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
        }).collect::<PersistenceResult<Vec<_>>>()?;

    let network_bridge_name = network_bridge_name.map(NetworkInterfaceName::try_from)
        .transpose()
        .map_err(|cause| error(Box::new(cause)))?;

    Ok(PeerDescriptor {
        id: peer_id,
        name,
        location,
        network: PeerNetworkDescriptor {
            interfaces: network_interfaces,
            bridge_name: network_bridge_name,
        },
        topology: Default::default(), //TODO
        executors: ExecutorDescriptors { executors: Default::default() }, //TODO
    })
}

#[cfg(test)]
mod tests {
    use opendut_types::peer::PeerId;
    use opendut_types::topology::Topology;
    use opendut_types::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    use crate::persistence::database;

    use super::*;

    #[tokio::test]
    async fn should_persist_peer_descriptor() -> anyhow::Result<()> {
        let mut db = database::testing::spawn_and_connect().await?;

        let peer_id = PeerId::random();

        let testee = PeerDescriptor {
            id: peer_id,
            name: PeerName::try_from("testee_name")?,
            location: Some(PeerLocation::try_from("testee_location")?),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: NetworkInterfaceId::random(),
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
                devices: vec![], //TODO
            },
            executors: ExecutorDescriptors { executors: vec![] }, //TODO
        };

        let result = get(peer_id, &mut db.connection)?;
        assert!(result.is_none());
        let result = list(&mut db.connection)?;
        assert!(result.is_empty());

        insert(testee.clone(), peer_id, &mut db.connection)?;

        let result = get(peer_id, &mut db.connection)?;
        assert_eq!(result, Some(testee.clone()));
        let result = list(&mut db.connection)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result.first(), Some(&testee));

        Ok(())
    }
}
