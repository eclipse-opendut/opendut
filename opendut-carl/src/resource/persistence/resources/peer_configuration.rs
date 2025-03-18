use opendut_types::peer::configuration::PeerConfiguration;
use opendut_types::peer::PeerId;

use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::persistence::Storage;
use crate::resource::storage::ResourcesStorageApi;

impl Persistable for PeerConfiguration {
    fn insert(self, id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.memory.insert(id, self)
    }

    fn remove(id: PeerId, storage: &mut Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.remove(id)
    }

    fn get(id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.get(id)
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        storage.memory.list()
    }
}
