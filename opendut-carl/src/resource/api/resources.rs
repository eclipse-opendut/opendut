use crate::resource::api::Resource;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::persistable::Persistable;
use crate::resource::storage::{ResourceTransaction, ResourcesStorageApi};
use crate::resource::subscription::{ResourceSubscriptionChannels, Subscribable, SubscriptionEvent};
use std::collections::HashMap;

pub type RelayedSubscriptionEvents = ResourceSubscriptionChannels;

pub struct Resources<'transaction> {
    transaction: ResourceTransaction<'transaction>,
}
impl<'transaction> Resources<'transaction> {
    pub fn new(transaction: ResourceTransaction<'transaction>) -> Self {
        Resources { transaction }
    }
}

fn notify_insertion_on_success<R>(event: SubscriptionEvent<R>, result: &PersistenceResult<()>, relayed_subscription_events: &mut RelayedSubscriptionEvents)
where R: Resource + Persistable + Subscribable {
    if result.is_ok() {
        let event_name = event.display_name().to_owned();
        relayed_subscription_events
            .notify(event)
            .unwrap_or_else(|_| panic!("should successfully queue notification about {event_name} resource during transaction"));
    }
}

fn notify_removal_on_success<R>(id: R::Id, result: &PersistenceResult<Option<R>>, relayed_subscription_events: &mut RelayedSubscriptionEvents)
where R: Resource + Persistable + Subscribable {
    if let Ok(Some(resource)) = result {
        let event = SubscriptionEvent::Removed { id, value: resource.clone() };
        let event_name = event.display_name().to_owned();
        relayed_subscription_events
            .notify(event)
            .unwrap_or_else(|_| panic!("should successfully queue notification about {event_name} resource during transaction"));

    }
}

impl ResourcesStorageApi for Resources<'_> {
    fn insert<R>(&mut self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable {
        let result = self.transaction.insert(id.clone(), resource.clone());
        notify_insertion_on_success(SubscriptionEvent::Inserted { id, value: resource }, &result, self.transaction.relayed_subscription_events);
        result
    }

    fn remove<R>(&mut self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Subscribable {
        let result = self.transaction.remove(id.clone());
        notify_removal_on_success(id, &result, self.transaction.relayed_subscription_events);
        result
    }

    fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        self.transaction.get(id)
    }

    fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where R: Resource + Persistable + Clone {
        self.transaction.list()
    }
}
