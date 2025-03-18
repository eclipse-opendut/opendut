use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::resource::Resource;
use crate::resource::storage::persistent::PersistentResourcesTransaction;
use crate::resource::storage::ResourcesStorageApi;
use crate::resource::storage::volatile::VolatileResourcesTransaction;
use crate::resource::subscription::{ResourceSubscriptionChannels, Subscribable, SubscriptionEvent};

pub type RelayedSubscriptionEvents = ResourceSubscriptionChannels;

pub enum ResourcesTransaction<'transaction> {
    Persistent(PersistentResourcesTransaction<'transaction>),
    Volatile(VolatileResourcesTransaction<'transaction>),
}
impl<'transaction> ResourcesTransaction<'transaction> {
    pub fn persistent(transaction: PersistentResourcesTransaction<'transaction>) -> Self {
        Self::Persistent(transaction)
    }
    pub fn volatile(transaction: VolatileResourcesTransaction<'transaction>) -> Self {
        Self::Volatile(transaction)
    }

    pub fn into_relayed_subscription_events(self) -> &'transaction mut RelayedSubscriptionEvents {
        match self {
            ResourcesTransaction::Persistent(transaction) => transaction.relayed_subscription_events,
            ResourcesTransaction::Volatile(transaction) => transaction.relayed_subscription_events,
        }
    }
}
impl ResourcesStorageApi for ResourcesTransaction<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable {
        match self {
            ResourcesTransaction::Persistent(transaction) => {
                let result = transaction.insert(id.clone(), resource.clone());
                if result.is_ok() {
                    transaction.relayed_subscription_events
                        .notify(SubscriptionEvent::Inserted { id, value: resource })
                        .expect("should successfully queue notification about resource insertion during transaction");
                }
                result
            }
            ResourcesTransaction::Volatile(transaction) => {
                let result = transaction.insert(id.clone(), resource.clone());
                if result.is_ok() {
                    transaction.relayed_subscription_events
                        .notify(SubscriptionEvent::Inserted { id, value: resource })
                        .expect("should successfully queue notification about resource insertion during transaction");
                }
                result
            }
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
        match &self {
            ResourcesTransaction::Persistent(transaction) => transaction.get(id),
            ResourcesTransaction::Volatile(transaction) => transaction.get(id),
        }
    }

    fn list<R>(&self) -> PersistenceResult<Vec<R>>
    where R: Resource + Persistable + Clone {
        match &self {
            ResourcesTransaction::Persistent(transaction) => transaction.list(),
            ResourcesTransaction::Volatile(transaction) => transaction.list(),
        }
    }
}
