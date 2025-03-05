use opendut_types::peer::PeerId;
use opendut_types::peer::state::PeerConnectionState;
use crate::persistence::error::PersistenceResult;
use crate::persistence::resources::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::ResourcesStorageApi;

impl Persistable for PeerConnectionState {
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