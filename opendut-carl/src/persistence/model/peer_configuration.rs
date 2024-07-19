use opendut_types::peer::configuration::PeerConfiguration;
use opendut_types::resources::Id;

use crate::persistence::model::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

impl Persistable for PeerConfiguration {
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
