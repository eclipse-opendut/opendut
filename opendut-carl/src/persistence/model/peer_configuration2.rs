use opendut_types::peer::configuration::PeerConfiguration2;
use opendut_types::peer::PeerId;

use crate::persistence::error::PersistenceResult;
use crate::persistence::model::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

impl Persistable for PeerConfiguration2 {
    fn insert(self, id: PeerId, storage: &mut Storage) -> PersistenceResult<()> {
        storage.memory.insert(id, self)
    }

    fn get(id: PeerId, storage: &Storage) -> PersistenceResult<Option<Self>> {
        storage.memory.get(id)
    }
    
    fn list(storage: &Storage) -> PersistenceResult<Vec<Self>> {
        storage.memory.list()
    }
}
