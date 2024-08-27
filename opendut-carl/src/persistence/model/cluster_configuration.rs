use super::{query, Persistable};
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::Filter;
use crate::persistence::Storage;
use diesel::Connection;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};

impl Persistable for ClusterConfiguration {
    fn insert(self, _id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.connection().transaction::<_, PersistenceError, _>(|connection| {
            query::cluster_configuration::insert(self, connection)
        })
    }

    fn remove(cluster_id: ClusterId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        query::cluster_configuration::remove(cluster_id, &mut storage.db.connection())
    }

    fn get(cluster_id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::cluster_configuration::list(Filter::By(cluster_id), &mut storage.db.connection())?
            .first().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::cluster_configuration::list(Filter::Not, &mut storage.db.connection())
    }
}
