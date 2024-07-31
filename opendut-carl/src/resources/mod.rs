use resource::Resource;

use crate::persistence::error::PersistenceResult;
use crate::persistence::model::Persistable;
use crate::resources::storage::{PersistenceOptions, ResourcesStorage, ResourcesStorageApi};

pub mod manager;
pub mod ids;
pub(crate) mod storage;
pub mod resource;

pub struct Resources {
    storage: ResourcesStorage,
}

impl Resources {
    pub fn connect(storage_options: PersistenceOptions) -> Result<Self, storage::ConnectionError> {
        let storage = ResourcesStorage::connect(storage_options)?;
        Ok(Self { storage })
    }

    /// Inserts a new resource with this ID or updates it, if it already exists.
    pub fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        match &mut self.storage {
            ResourcesStorage::Persistent(storage) => storage.insert(id, resource),
            ResourcesStorage::Volatile(storage) => storage.insert(id, resource),
        }
    }

    pub fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        match &mut self.storage {
            ResourcesStorage::Persistent(storage) => storage.remove(id),
            ResourcesStorage::Volatile(storage) => storage.remove(id),
        }
    }

    pub fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        match &self.storage {
            ResourcesStorage::Persistent(storage) => storage.get(id),
            ResourcesStorage::Volatile(storage) => storage.get(id),
        }
    }

    pub fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable {
        match &self.storage {
            ResourcesStorage::Persistent(storage) => storage.list(),
            ResourcesStorage::Volatile(storage) => storage.list(),
        }
    }
}

#[cfg(test)]
impl Resources {
    pub async fn contains<R>(&self, id: R::Id) -> bool
    where R: Resource {
        match &self.storage {
            ResourcesStorage::Persistent(_) => unimplemented!(),
            ResourcesStorage::Volatile(storage) => storage.contains::<R>(id),
        }
    }

    pub async fn is_empty(&self) -> bool {
        match &self.storage {
            ResourcesStorage::Persistent(_) => unimplemented!(),
            ResourcesStorage::Volatile(storage) => storage.is_empty(),
        }
    }
}
