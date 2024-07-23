use opendut_types::cluster::ClusterConfiguration;
use opendut_types::resources::Id;

use crate::persistence::error::PersistenceResult;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

use super::Persistable;

impl Persistable for ClusterConfiguration {
    fn insert(self, id: Id, storage: &mut Storage) -> PersistenceResult<()> {
        storage.memory.insert(id, self)
    }

    fn get(id: Id, storage: &Storage) -> PersistenceResult<Option<Self>> {
        Ok(storage.memory.get(id))
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        Ok(storage.memory.list())
    }
}
