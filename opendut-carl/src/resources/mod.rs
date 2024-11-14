use crate::persistence::error::PersistenceResult;
use crate::persistence::resources::Persistable;
use crate::resources::storage::{PersistenceOptions, ResourcesStorage, ResourcesStorageApi};
use crate::resources::subscription::Subscribable;
use crate::resources::transaction::{RelayedSubscriptionEvents, ResourcesTransaction};
use resource::Resource;

pub mod manager;
pub mod ids;
pub mod resource;
pub(crate) mod storage;
pub(crate) mod subscription;
mod transaction;

pub struct Resources {
    storage: ResourcesStorage,
}

impl Resources {
    pub async fn connect(storage_options: PersistenceOptions) -> Result<Self, storage::ConnectionError> {
        let storage = ResourcesStorage::connect(storage_options).await?;
        Ok(Self { storage })
    }

    pub(super) fn transaction<T, E, F>(&mut self, code: F) -> PersistenceResult<(Result<T, E>, RelayedSubscriptionEvents)>
    where
        F: FnOnce(&mut ResourcesTransaction) -> Result<T, E>,
        E: Send + Sync + 'static,
    {
        match &mut self.storage {
            ResourcesStorage::Persistent(storage) => storage.transaction(|transaction| {
                let mut transaction = ResourcesTransaction::persistent(transaction);
                code(&mut transaction)
            }),
            ResourcesStorage::Volatile(storage) => storage.noop_transaction(|transaction| {
                let mut transaction = ResourcesTransaction::volatile(transaction);
                code(&mut transaction)
            }),
        }
    }
}

impl ResourcesStorageApi for Resources {
    /// Inserts a new resource with this ID or updates it, if it already exists.
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable {
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
