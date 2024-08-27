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

    fn remove(cluster_id: ClusterId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        query::cluster_configuration::remove(cluster_id, storage.db.lock().unwrap().deref_mut())
    }

    fn get(cluster_id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::cluster_configuration::list(Filter::By(cluster_id), storage.db.lock().unwrap().deref_mut())?
            .first().cloned();
        Ok(result)
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::cluster_configuration::list(Filter::Not, storage.db.lock().unwrap().deref_mut())
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
