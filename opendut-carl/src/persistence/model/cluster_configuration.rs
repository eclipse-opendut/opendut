use super::{query, Persistable};
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::cluster_configuration::PersistableClusterConfiguration;
use crate::persistence::model::query::cluster_device::PersistableClusterDevice;
use crate::persistence::model::query::Filter;
use crate::persistence::Storage;
use diesel::{Connection, PgConnection};
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use std::ops::DerefMut;

impl Persistable for ClusterConfiguration {
    fn insert(self, _id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(id: ClusterId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        remove_from_database(id, storage.db.lock().unwrap().deref_mut())
    }

    fn get(id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        get_from_database(id, storage.db.lock().unwrap().deref_mut())
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        list_database(storage.db.lock().unwrap().deref_mut())
    }
}

pub(super) fn insert_into_database(cluster_configuration: ClusterConfiguration, connection: &mut PgConnection) -> PersistenceResult<()> {
    let ClusterConfiguration { id, name, leader, devices } = cluster_configuration;

    query::cluster_configuration::insert(PersistableClusterConfiguration {
        cluster_id: id.0,
        name: name.value(),
        leader_id: leader.uuid,
    }, connection)?;

    for device in devices {
        query::cluster_device::insert(PersistableClusterDevice {
            cluster_id: id.0,
            device_id: device.0,
        }, connection)?
    }

    Ok(())
}

fn remove_from_database(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterConfiguration>> {
    query::cluster_configuration::remove(cluster_id, connection)
}

fn get_from_database(cluster_id: ClusterId, connection: &mut PgConnection) -> PersistenceResult<Option<ClusterConfiguration>> {
    let result = query::cluster_configuration::list(Filter::By(cluster_id), connection)?
        .first().cloned();
    Ok(result)
}

fn list_database(connection: &mut PgConnection) -> PersistenceResult<Vec<ClusterConfiguration>> {
    query::cluster_configuration::list(Filter::Not, connection)
}


#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::persistence::database;
    use opendut_types::cluster::ClusterName;
    use opendut_types::peer::PeerId;
    use opendut_types::topology::DeviceId;
    use std::collections::HashSet;

    #[tokio::test]
    async fn should_persist_cluster_configuration() -> anyhow::Result<()> {
        let mut db = database::testing::spawn_and_connect().await?;

        let peer = crate::persistence::model::peer_descriptor::tests::peer_descriptor()?;
        crate::persistence::model::peer_descriptor::insert_into_database(peer.clone(), &mut db.connection)?;

        let testee = cluster_configuration(
            peer.id,
            peer.topology.devices.into_iter().map(|device| device.id).collect()
        )?;

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

    pub fn cluster_configuration(leader_id: PeerId, devices: Vec<DeviceId>) -> anyhow::Result<ClusterConfiguration> {
        Ok(ClusterConfiguration {
            id: ClusterId::random(),
            name: ClusterName::try_from("cluster-name")?,
            leader: leader_id,
            devices: HashSet::from_iter(devices),
        })
    }
}
