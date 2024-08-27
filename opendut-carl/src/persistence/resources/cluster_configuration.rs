use super::Persistable;
use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::query::Filter;
use crate::persistence::{query, Storage};
use diesel::Connection;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};

impl Persistable for ClusterConfiguration {
    fn insert(self, _id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.connection().transaction::<_, PersistenceError, _>(|connection| {
            //Delete before inserting to ensure that when an update removes
            //list elements we don't leave those elements behind in the database.
            //TODO more efficient solution
            query::cluster_configuration::remove(self.id, connection)?;

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
