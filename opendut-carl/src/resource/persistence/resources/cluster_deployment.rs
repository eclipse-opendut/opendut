use std::collections::HashMap;
use opendut_types::cluster::{ClusterDeployment, ClusterId};

use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::query::Filter;
use crate::resource::persistence::{query, Storage};

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
        let result = query::cluster_deployment::list(Filter::By(cluster_id), &mut storage.db.connection())?.values().next().cloned();
        Ok(result)
    }

    fn list(storage: &Storage) -> PersistenceResult<HashMap<Self::Id, Self>> {
        query::cluster_deployment::list(Filter::Not, &mut storage.db.connection())
    }
}
