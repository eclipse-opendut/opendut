use opendut_types::peer::configuration::PeerConfiguration2;
use opendut_types::resources::Id;

use crate::persistence::model::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

impl Persistable for PeerConfiguration2 {
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
