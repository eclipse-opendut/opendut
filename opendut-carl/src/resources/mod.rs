use resource::Resource;

use crate::persistence::error::PersistenceResult;
use crate::persistence::resources::Persistable;
use crate::resources::storage::persistent::PersistentResourcesTransaction;
use crate::resources::storage::volatile::VolatileResourcesTransaction;
use crate::resources::storage::{PersistenceOptions, ResourcesStorage, ResourcesStorageApi};

pub mod manager;
pub mod ids;
pub(crate) mod storage;
pub mod resource;

pub struct Resources {
    storage: ResourcesStorage,
}

impl Resources {
    pub async fn connect(storage_options: PersistenceOptions) -> Result<Self, storage::ConnectionError> {
        let storage = ResourcesStorage::connect(storage_options).await?;
        Ok(Self { storage })
    }

    pub fn transaction<T, E, F>(&mut self, code: F) -> PersistenceResult<Result<T, E>>
    where
        F: FnOnce(ResourcesTransaction) -> Result<T, E>,
        E: std::error::Error + Send + Sync + 'static,
    {
        match &mut self.storage {
            ResourcesStorage::Persistent(storage) => storage.transaction(|transaction| code(ResourcesTransaction::Persistent(transaction))),
            ResourcesStorage::Volatile(storage) => storage.noop_transaction(|transaction| code(ResourcesTransaction::Volatile(transaction))),
        }
    }
}
impl ResourcesStorageApi for Resources {
    /// Inserts a new resource with this ID or updates it, if it already exists.
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        match &mut self.storage {
            ResourcesStorage::Persistent(storage) => storage.insert(id, resource),
            ResourcesStorage::Volatile(storage) => storage.insert(id, resource),
        }
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        match &mut self.storage {
            ResourcesStorage::Persistent(storage) => storage.remove(id),
            ResourcesStorage::Volatile(storage) => storage.remove(id),
        }
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        match &self.storage {
            ResourcesStorage::Persistent(storage) => storage.get(id),
            ResourcesStorage::Volatile(storage) => storage.get(id),
        }
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
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

pub enum ResourcesTransaction<'a> {
    Persistent(PersistentResourcesTransaction<'a>),
    Volatile(VolatileResourcesTransaction<'a>),
}
impl ResourcesStorageApi for ResourcesTransaction<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable {
        match self {
            ResourcesTransaction::Persistent(transaction) => transaction.insert(id, resource),
            ResourcesTransaction::Volatile(transaction) => transaction.insert(id, resource),
        }
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        match self {
            ResourcesTransaction::Persistent(transaction) => transaction.remove(id),
            ResourcesTransaction::Volatile(transaction) => transaction.remove(id),
        }
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        match self {
            ResourcesTransaction::Persistent(transaction) => transaction.get(id),
            ResourcesTransaction::Volatile(transaction) => transaction.get(id),
        }
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable + Clone {
        match self {
            ResourcesTransaction::Persistent(transaction) => transaction.list(),
            ResourcesTransaction::Volatile(transaction) => transaction.list(),
        }
    }
}
