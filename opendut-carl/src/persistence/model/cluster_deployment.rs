use opendut_types::cluster::ClusterDeployment;
use opendut_types::resources::Id;

use crate::persistence::error::PersistenceResult;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

use super::Persistable;

impl Persistable for ClusterDeployment {
    fn insert(self, id: Id, storage: &mut Storage) -> PersistenceResult<()> {
        storage.memory.insert(id, self)
    }

    fn get(id: Id, storage: &Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.get(id)
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        storage.memory.list()
    }
}
