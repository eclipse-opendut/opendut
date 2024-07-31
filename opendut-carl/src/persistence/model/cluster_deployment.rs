use opendut_types::cluster::{ClusterDeployment, ClusterId};

use crate::persistence::error::PersistenceResult;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

use super::Persistable;

impl Persistable for ClusterDeployment {
    fn insert(self, id: ClusterId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.memory.insert(id, self)
    }

    fn remove(id: Self::Id, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.remove(id)
    }

    fn get(id: ClusterId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.get(id)
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        storage.memory.list()
    }
}
