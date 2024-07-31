use std::sync::Mutex;
use url::Url;

use crate::persistence::database::ConnectError;
use crate::persistence::error::PersistenceResult;
use crate::persistence::model::Persistable;
use crate::persistence::Storage;
use crate::resources::storage::{Resource, ResourcesStorageApi};
use crate::resources::storage::volatile::VolatileResourcesStorage;

pub struct PersistentResourcesStorage {
    storage: Storage,
}
impl PersistentResourcesStorage {
    pub fn connect(url: &Url) -> Result<Self, ConnectError> {
        let db = Mutex::new(
            crate::persistence::database::connect(url)?
        );
        let memory = VolatileResourcesStorage::default();
        let storage = Storage { db, memory };
        Ok(Self { storage })
    }
}
impl ResourcesStorageApi for PersistentResourcesStorage {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        resource.insert(id, &mut self.storage)
    }

    fn remove<R>(&mut self, id: R::Id) -> Option<R>
    where R: Resource + Persistable {
        todo!()
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        R::get(id, &self.storage)
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable + Clone {
        R::list(&self.storage)
    }
}
