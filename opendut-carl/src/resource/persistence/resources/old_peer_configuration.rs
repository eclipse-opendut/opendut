use opendut_types::peer::configuration::OldPeerConfiguration;
use opendut_types::peer::PeerId;
use std::collections::HashMap;

use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::persistence::{Db, Memory};
use crate::resource::storage::ResourcesStorageApi;

impl Persistable for OldPeerConfiguration {
    fn insert(self, id: PeerId, memory: &mut Memory, _: &Db) -> PersistenceResult<()> {
        memory.lock().unwrap().insert(id, self)
    }

    fn remove(id: PeerId, memory: &mut Memory, _: &Db) -> PersistenceResult<Option<Self>> {
        memory.lock().unwrap().remove(id)
    }

    fn get(id: PeerId, memory: &Memory, _: &Db) -> PersistenceResult<Option<Self>> {
        memory.lock().unwrap().get(id)
    }
    
    fn list(memory: &Memory, _: &Db) -> PersistenceResult<HashMap<Self::Id, Self>> {
        memory.lock().unwrap().list()
    }
}
