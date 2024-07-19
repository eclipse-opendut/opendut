use opendut_types::peer::state::PeerState;
use opendut_types::resources::Id;

use crate::persistence::model::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

impl Persistable for PeerState {
    fn insert(self, id: Id, storage: &mut Storage) {
        storage.memory.insert(id, self)
    }

    fn get(id: Id, storage: &Storage) -> Option<Self> {
        storage.memory.get(id)
    }
    
    fn list(storage: &Storage) -> Vec<Self> {
        storage.memory.list()
    }
}
