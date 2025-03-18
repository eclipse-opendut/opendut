use std::any::{Any, TypeId};
use std::collections::HashMap;

use opendut_types::resources::Id;

use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::ids::IntoId;
use crate::resource::storage::ResourcesStorageApi;
use crate::resource::Resource;
use crate::resource::subscription::Subscribable;
use crate::resource::transaction::RelayedSubscriptionEvents;

#[derive(Default)]
pub struct VolatileResourcesStorage {
    storage: HashMap<TypeId, HashMap<Id, Box<dyn Any + Send + Sync>>>,
}
impl VolatileResourcesStorage {
    pub fn noop_transaction<T, E, F>(&mut self, code: F) -> PersistenceResult<(Result<T, E>, RelayedSubscriptionEvents)>
    where
        F: FnOnce(VolatileResourcesTransaction) -> Result<T, E>,
        E: Send + Sync + 'static,
    {
        let mut relayed_subscription_events = RelayedSubscriptionEvents::default();
        let transaction = VolatileResourcesTransaction {
            inner: self,
            relayed_subscription_events: &mut relayed_subscription_events,
        };
        let result = code(transaction);
        Ok((result, relayed_subscription_events))
    }
}

impl ResourcesStorageApi for VolatileResourcesStorage {

    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource {
        let id = id.into_id();
        let column = self.storage
            .entry(TypeId::of::<R>())
            .or_default();
        column.insert(id, Box::new(resource));
        Ok(())
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource {
        let id = id.into_id();
        let type_id = TypeId::of::<R>();
        match self.column_mut_of::<R>() {
            None => Ok(None),
            Some(column) => {
                let result = column.remove(&id)
                    .and_then(|old_value| old_value
                        .downcast()
                        .map(|value| *value)
                        .ok()
                    );
                if column.is_empty() {
                    self.storage.remove(&type_id);
                }
                Ok(result)
            }
        }
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Clone {
        let id = id.into_id();
        let result = self.column_of::<R>()
            .and_then(|column| column.get(&id))
            .and_then(|resource| resource.downcast_ref().cloned());
        Ok(result)
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource {
        let result = match self.column_of::<R>() {
            Some(column) => {
                column.values()
                    .map(|value| value
                        .downcast_ref::<R>()
                        .cloned()
                        .expect("It should always be possible to cast the stored data back to its own type while building an iterator.")
                    )
                    .collect()
            }
            None => Vec::new()
        };
        Ok(result)
    }
}
impl VolatileResourcesStorage {
    fn column_of<R>(&self) -> Option<&HashMap<Id, Box<dyn Any + Send + Sync>>>
    where R: Resource {
        self.storage.get(&TypeId::of::<R>())
    }

    fn column_mut_of<R>(&mut self) -> Option<&mut HashMap<Id, Box<dyn Any + Send + Sync>>>
    where R: Resource {
        self.storage.get_mut(&TypeId::of::<R>())
    }
}

#[cfg(test)]
impl VolatileResourcesStorage {
    pub fn contains<R>(&self, id: R::Id) -> bool
    where R: Resource {
        let id = id.into_id();
        if let Some(column) = self.column_of::<R>() {
            column.contains_key(&id)
        }
        else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}

pub struct VolatileResourcesTransaction<'transaction> {
    inner: &'transaction mut VolatileResourcesStorage,
    pub relayed_subscription_events: &'transaction mut RelayedSubscriptionEvents,
}
impl ResourcesStorageApi for VolatileResourcesTransaction<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable {
        self.inner.insert(id, resource)
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable {
        self.inner.remove(id)
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        self.inner.get(id)
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable + Clone {
        self.inner.list()
    }
}
