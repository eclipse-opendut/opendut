use opendut_types::peer::configuration::OldPeerConfiguration;
use opendut_types::peer::PeerId;
use std::collections::HashMap;

use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::persistence::Storage;
use crate::resource::storage::ResourcesStorageApi;

impl Persistable for OldPeerConfiguration {
    fn insert(self, id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.memory.insert(id, self)
    }

    fn remove(id: PeerId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.remove(id)
    }

    fn get(id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.get(id)
    }
    
    fn list(storage: &Storage) -> PersistenceResult<HashMap<Self::Id, Self>> {
        storage.memory.list()
    }
}
