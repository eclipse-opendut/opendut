use opendut_types::resources::Id;
use opendut_types::topology::DeviceDescriptor;

use crate::persistence::error::PersistenceResult;
use crate::persistence::model::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

impl Persistable for DeviceDescriptor {
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
