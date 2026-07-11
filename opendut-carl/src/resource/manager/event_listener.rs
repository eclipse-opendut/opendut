use anyhow::anyhow;
use tracing::{debug, warn, error};

use crate::resource::{manager::{ResourceManager, SubscriptionEvent}, types::{Resource, subscription::{self, Subscribable, Subscription}}};


pub(super) async fn spawn_event_listener<R: Resource + Subscribable>(
    resource_manager: &ResourceManager,
    on_event: impl AsyncFn(SubscriptionEvent<R>) + Send + 'static,
) {
    let resource_name = R::type_name();
    let mut subscription = resource_manager.subscribe::<R>().await;
    let cancel = resource_manager.event_listener_cancel.clone();

    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {
            loop {
                tokio::select! {
                    result = receive(&mut subscription, resource_name) => {
                        if let Some(event) = result? {
                            on_event(event).await;
                        }
                    },
                    _ = cancel.cancelled() => {
                        debug!("ResourceManager is cancelling subscription channel for {resource_name}.");
                        break;
                    }
                }
            }

            anyhow::Ok(())
        })
    });
}


pub async fn spawn_event_listener_aggregate_2<R1: Resource + Subscribable, R2: Resource + Subscribable>(
    resource_manager: &ResourceManager,
    on_event: impl AsyncFn((Option<SubscriptionEvent<R1>>, Option<SubscriptionEvent<R2>>)) + Send + 'static,
) {

    let mut subscription1 = resource_manager.subscribe::<R1>().await;
    let mut subscription2 = resource_manager.subscribe::<R2>().await;

    let aggregate_resource_name = format!(
        "<{}>",
        [
            R1::type_name(),
            R2::type_name(),
        ].join(", ")
    );


    let cancel = resource_manager.event_listener_cancel.clone();


    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async move {

            let mut previous1 = None;
            let mut previous2 = None;

            loop {
                tokio::select! {
                    result = receive(&mut subscription1, &aggregate_resource_name) => {
                        if let Some(event) = result? {
                            previous1.replace(event);

                            let notification = (previous1.clone(), previous2.clone());
                            on_event(notification).await;
                        }
                    },
                    result = receive(&mut subscription2, &aggregate_resource_name) => {
                        if let Some(event) = result? {
                            previous2.replace(event);

                            let notification = (previous1.clone(), previous2.clone());
                            on_event(notification).await;
                        }
                    },
                    _ = cancel.cancelled() => {
                        debug!("ResourceManager is cancelling subscription channel for {aggregate_resource_name}.");
                        break;
                    }
                }
            }

            anyhow::Ok(())
        })
    });
}


async fn receive<R: Resource>(
    subscription: &mut Subscription<R>,
    aggregate_resource_name: &str,
) -> anyhow::Result<Option<SubscriptionEvent<R>>> {
    let resource_name = R::type_name();

    match subscription.receive().await {
        Ok(event) => Ok(Some(event)),
        Err(subscription::ReceiveError::Broadcast(error)) => match error {
            tokio::sync::broadcast::error::RecvError::Closed => {
                let message = format!("Channel for {resource_name} in ResourceManager subscription {aggregate_resource_name} closed. Aborting.");
                error!("{message}");
                Err(anyhow!(message))
            }
            tokio::sync::broadcast::error::RecvError::Lagged(skipped_messages) => {
                warn!("Channel for {resource_name} in ResourceManager subscription {aggregate_resource_name} lagged behind and had to skip {skipped_messages} messages.");
                Ok(None)
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use opendut_model::{cluster::{ClusterDeployment, ClusterId}, peer::{PeerId, state::PeerConnectionState}};
    use tokio::sync::mpsc;


    #[tokio::test]
    async fn should_spawn_an_event_listener() -> anyhow::Result<()> {
        let (testee, _cancel) = ResourceManager::new_in_memory();

        let (sender, mut receiver) = mpsc::channel(1);

        testee.spawn_event_listener::<ClusterDeployment>(async move |_event| {
            sender.send(()).await.unwrap();
        }).await;

        let id = ClusterId::random();
        testee.insert(id, ClusterDeployment { id }).await?;

        let received = tokio::time::timeout(
            Duration::from_secs(5),
            receiver.recv()
        ).await?;

        assert!(received.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn should_spawn_an_event_listener_that_aggregates_two_resources() -> anyhow::Result<()> {
        let (testee, _cancel) = ResourceManager::new_in_memory();

        let (sender, mut receiver) = mpsc::channel(1);

        testee.spawn_event_listener_aggregate_2::<ClusterDeployment, PeerConnectionState>(
            async move |event| {
                sender.send(event).await.unwrap();
            }
        ).await;



        let id1 = ClusterId::random();
        let resource1 = ClusterDeployment { id: id1 };
        testee.insert(id1, resource1.clone()).await?;

        {
            let received = tokio::time::timeout(
                Duration::from_secs(5),
                receiver.recv()
            ).await?;

            let (received1, received2) = received.unwrap();

            assert_eq!(received1, Some(SubscriptionEvent::Inserted { id: id1, value: resource1.clone() }));
            assert_eq!(received2, None);
        }


        let id2 = PeerId::random();
        let resource2 = PeerConnectionState::Offline;
        testee.insert(id2, resource2.clone()).await?;

        {
            let received = tokio::time::timeout(
                Duration::from_secs(5),
                receiver.recv()
            ).await?;

            let (received1, received2) = received.unwrap();

            assert_eq!(received1, Some(SubscriptionEvent::Inserted { id: id1, value: resource1 }));
            assert_eq!(received2, Some(SubscriptionEvent::Inserted { id: id2, value: resource2 }));
        }
        Ok(())
    }
}
