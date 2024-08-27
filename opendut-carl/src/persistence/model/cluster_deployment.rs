use diesel::{Connection, PgConnection};
use opendut_types::cluster::{ClusterDeployment, ClusterId};
use std::ops::DerefMut;

use crate::persistence::error::{PersistenceError, PersistenceResult};
use crate::persistence::model::query::Filter;
use crate::persistence::Storage;

use super::{query, Persistable};

impl Persistable for ClusterDeployment {
    fn insert(self, _id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.db.lock().unwrap().transaction::<_, PersistenceError, _>(|connection| {
            insert_into_database(self, connection)
        })
    }

    fn remove(cluster_id: ClusterId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        query::cluster_deployment::remove(cluster_id, storage.db.lock().unwrap().deref_mut())
    }

    fn get(cluster_id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::cluster_deployment::list(Filter::By(cluster_id), storage.db.lock().unwrap().deref_mut())?
            .first().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::cluster_deployment::list(Filter::Not, storage.db.lock().unwrap().deref_mut())
    }
}


fn insert_into_database(cluster_deployment: ClusterDeployment, connection: &mut PgConnection) -> PersistenceResult<()> {
    let ClusterDeployment { id } = cluster_deployment;

    query::cluster_deployment::insert(id, connection)?;

    Ok(())
}
