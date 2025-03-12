use opendut_types::cluster::{ClusterDeployment, ClusterId};

use crate::persistence::error::PersistenceResult;
use crate::persistence::query::Filter;
use crate::persistence::{query, Storage};

use super::Persistable;

impl Persistable for ClusterDeployment {
    fn insert(self, _id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {
        let mut connection = storage.db.connection();

        query::cluster_deployment::insert(self, &mut connection)
    }

    fn remove(cluster_id: ClusterId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        query::cluster_deployment::remove(cluster_id, &mut storage.db.connection())
    }

    fn get(cluster_id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        let result = query::cluster_deployment::list(Filter::By(cluster_id), &mut storage.db.connection())?
            .first().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        query::cluster_deployment::list(Filter::Not, &mut storage.db.connection())
    }
}
