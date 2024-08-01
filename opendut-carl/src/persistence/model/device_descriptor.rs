use opendut_types::topology::{DeviceDescriptor, DeviceId};

use crate::persistence::error::PersistenceResult;
use crate::persistence::model::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

impl Persistable for DeviceDescriptor {
    fn insert(self, id: DeviceId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.memory.insert(id, self)
    }

    fn remove(id: DeviceId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.remove(id)
    }

    fn get(id: DeviceId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.get(id)
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        storage.memory.list()
    }
}
