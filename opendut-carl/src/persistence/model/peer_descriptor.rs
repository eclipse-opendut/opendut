use diesel::{Connection, PgConnection};
use std::ops::DerefMut;

use opendut_types::peer::{PeerDescriptor, PeerId, PeerNetworkDescriptor};
use opendut_types::topology::Topology;

use super::{query, Persistable};
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::peer_descriptor::PersistablePeerDescriptor;
use crate::persistence::model::query::Filter;
use crate::persistence::Storage;

impl Persistable for PeerDescriptor {
    fn insert(self, _peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(peer_id: PeerId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        remove_from_database(peer_id, storage.db.lock().unwrap().deref_mut())
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

    query::peer_descriptor::insert(PersistablePeerDescriptor {
        peer_id: peer_id.uuid,
        name: name.value(),
        location: location.map(|location| location.value()),
        network_bridge_name: bridge_name.map(|name| name.name()),
    }, connection)?;

    for interface in interfaces {
        query::network_interface_descriptor::insert_into_database(&interface, peer_id, connection)?;
    }

    let Topology { devices } = topology;

    for device in devices {
        crate::persistence::model::device_descriptor::insert_into_database(device, connection)?;
    }

    // TODO persist executors

    Ok(())
}

fn remove_from_database(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Option<PeerDescriptor>> {
    query::peer_descriptor::remove(peer_id, connection)
}

fn get_from_database(peer_id: PeerId, connection: &mut PgConnection) -> PersistenceResult<Option<PeerDescriptor>> {
    let result = query::peer_descriptor::list(Filter::By(peer_id), connection)?
        .first().cloned();
    Ok(result)
}

fn list_database(connection: &mut PgConnection) -> PersistenceResult<Vec<PeerDescriptor>> {
    query::peer_descriptor::list(Filter::Not, connection)
}


#[cfg(test)]
pub(super) mod tests {
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::peer::{PeerId, PeerLocation, PeerName};
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag, Topology};
    use opendut_types::util::net::{CanSamplePoint, NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    use super::*;
    use crate::persistence::database;

    #[tokio::test]
    async fn should_persist_peer_descriptor() -> anyhow::Result<()> {
        let mut db = database::testing::spawn_and_connect().await?;

        let testee = peer_descriptor()?;

        let result = get_from_database(testee.id, &mut db.connection)?;
        assert!(result.is_none());
        let result = list_database(&mut db.connection)?;
        assert!(result.is_empty());

        insert_into_database(testee.clone(), &mut db.connection)?;

        let result = get_from_database(testee.id, &mut db.connection)?;
        assert_eq!(result, Some(testee.clone()));
        let result = list_database(&mut db.connection)?;
        assert_eq!(result.len(), 1);
        assert_eq!(result.first(), Some(&testee));

        let result = remove_from_database(testee.id, &mut db.connection)?;
        assert_eq!(result, Some(testee.clone()));

        let result = get_from_database(testee.id, &mut db.connection)?;
        assert!(result.is_none());
        let result = list_database(&mut db.connection)?;
        assert!(result.is_empty());

        let result = remove_from_database(testee.id, &mut db.connection)?;
        assert_eq!(result, None);

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
