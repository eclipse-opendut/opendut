use crate::resource::api::Resource;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::storage::persistent::PersistentResourcesTransaction;
use crate::resource::storage::volatile::VolatileResourcesTransaction;
use crate::resource::storage::ResourcesStorageApi;
use crate::resource::subscription::{ResourceSubscriptionChannels, Subscribable, SubscriptionEvent};
use std::collections::HashMap;
use crate::resource::api::global::GlobalResourcesRef;

pub type RelayedSubscriptionEvents = ResourceSubscriptionChannels;

pub struct Resources<'transaction> {
    kind: ResourcesKind<'transaction>,
    pub global: GlobalResourcesRef,
}
pub enum ResourcesKind<'transaction> {
    Persistent(PersistentResourcesTransaction<'transaction>),
    Volatile(VolatileResourcesTransaction<'transaction>),
}
impl<'transaction> Resources<'transaction> {
    pub fn persistent(transaction: PersistentResourcesTransaction<'transaction>, global: GlobalResourcesRef) -> Self {
        Resources {
            kind: ResourcesKind::Persistent(transaction),
            global,
        }
    }
    pub fn volatile(transaction: VolatileResourcesTransaction<'transaction>, global: GlobalResourcesRef) -> Self {
        Resources {
            kind: ResourcesKind::Volatile(transaction),
            global,
        }
    }

    pub fn into_relayed_subscription_events(self) -> &'transaction mut RelayedSubscriptionEvents {
        match self.kind {
            ResourcesKind::Persistent(transaction) => transaction.relayed_subscription_events,
            ResourcesKind::Volatile(transaction) => transaction.relayed_subscription_events,
        }
    }
}
impl ResourcesStorageApi for Resources<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable {
        match &mut self.kind {
            ResourcesKind::Persistent(transaction) => {
                let result = transaction.insert(id.clone(), resource.clone());
                if result.is_ok() {
                    transaction.relayed_subscription_events
                        .notify(SubscriptionEvent::Inserted { id, value: resource })
                        .expect("should successfully queue notification about resource insertion during transaction");
                }
                result
            }
            ResourcesKind::Volatile(transaction) => {
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
        //TODO notify subscription events
        match &mut self.kind {
            ResourcesKind::Persistent(transaction) => transaction.remove(id),
            ResourcesKind::Volatile(transaction) => transaction.remove(id),
        }
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        match &self.kind {
            ResourcesKind::Persistent(transaction) => transaction.get(id),
            ResourcesKind::Volatile(transaction) => transaction.get(id),
        }
    }

    fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where R: Resource + Persistable + Clone {
        match &self.kind {
            ResourcesKind::Persistent(transaction) => transaction.list(),
            ResourcesKind::Volatile(transaction) => transaction.list(),
        }
    }
}
