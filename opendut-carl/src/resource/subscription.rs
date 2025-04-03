use crate::resource::api::Resource;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use opendut_types::peer::state::PeerConnectionState;
use opendut_types::peer::PeerDescriptor;
use tokio::sync::broadcast;


pub struct Subscription<R: Resource> {
    receiver: broadcast::Receiver<SubscriptionEvent<R>>,
}
impl<R: Resource> Subscription<R> {
    pub async fn receive(&mut self) -> Result<SubscriptionEvent<R>, ReceiveError> {
        self.receiver.recv().await
            .map_err(ReceiveError::Broadcast)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubscriptionEvent<R: Resource> {
    ///Resource was created or updated.
    Inserted { id: R::Id, value: R },
    Removed { id: R::Id, value: R },
}
impl<R: Resource> SubscriptionEvent<R> {
    pub fn display_name(&self) -> &str {
        match self {
            SubscriptionEvent::Inserted { .. } => { "inserted" }
            SubscriptionEvent::Removed { .. } => { "removed" }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ReceiveError {
    #[error("Error while receiving Broadcast")]
    Broadcast(#[from] broadcast::error::RecvError),
}

pub trait Subscribable: Resource {
    fn resource_subscribers_field(resource_subscribers: &mut ResourceSubscriptionChannels) -> &broadcast::Sender<SubscriptionEvent<Self>>
    where Self: Sized;
}
macro_rules! impl_subscribable {
    ($resource:ty, $field:ident) => {
        impl Subscribable for $resource {
            fn resource_subscribers_field(resource_subscribers: &mut ResourceSubscriptionChannels) -> &broadcast::Sender<SubscriptionEvent<Self>>
            where Self: Sized {
                let (sender, _) = &resource_subscribers.$field;
                sender
            }
        }
    }
}
impl_subscribable!(ClusterConfiguration, cluster_configuration);
impl_subscribable!(ClusterDeployment, cluster_deployment);
impl_subscribable!(OldPeerConfiguration, old_peer_configuration);
impl_subscribable!(PeerConfiguration, peer_configuration);
impl_subscribable!(PeerDescriptor, peer_descriptor);
impl_subscribable!(PeerConnectionState, peer_connection_state);


pub type ResourceSubscriptionChannel<R> = (broadcast::Sender<SubscriptionEvent<R>>, broadcast::Receiver<SubscriptionEvent<R>>); //store both the sender and initial receiver, to prevent channel from closing

pub struct ResourceSubscriptionChannels {
    pub cluster_configuration: ResourceSubscriptionChannel<ClusterConfiguration>,
    pub cluster_deployment: ResourceSubscriptionChannel<ClusterDeployment>,
    pub old_peer_configuration: ResourceSubscriptionChannel<OldPeerConfiguration>,
    pub peer_configuration: ResourceSubscriptionChannel<PeerConfiguration>,
    pub peer_descriptor: ResourceSubscriptionChannel<PeerDescriptor>,
    pub peer_connection_state: ResourceSubscriptionChannel<PeerConnectionState>,
}
impl ResourceSubscriptionChannels {
    pub fn subscribe<R: Resource + Subscribable>(&mut self) -> Subscription<R> {
        let receiver = R::resource_subscribers_field(self)
            .subscribe();
        Subscription { receiver }
    }

    pub fn notify<R: Resource + Subscribable + Sized>(&mut self, event: SubscriptionEvent<R>) -> Result<(), broadcast::error::SendError<SubscriptionEvent<R>>> {
        R::resource_subscribers_field(self)
            .send(event)?;
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        let ResourceSubscriptionChannels { cluster_configuration, cluster_deployment, old_peer_configuration, peer_configuration, peer_descriptor, peer_connection_state } = self;

        cluster_configuration.0.is_empty()
        && cluster_deployment.0.is_empty()
        && old_peer_configuration.0.is_empty()
        && peer_configuration.0.is_empty()
        && peer_descriptor.0.is_empty()
        && peer_connection_state.0.is_empty()
    }
}
impl Default for ResourceSubscriptionChannels {
    fn default() -> Self {
        let capacity = 100;

        let cluster_configuration = broadcast::channel(capacity);
        let cluster_deployment = broadcast::channel(capacity);
        let old_peer_configuration = broadcast::channel(capacity);
        let peer_configuration = broadcast::channel(capacity);
        let peer_descriptor = broadcast::channel(capacity);
        let peer_connection_state = broadcast::channel(capacity);

        Self {
            cluster_configuration,
            cluster_deployment,
            old_peer_configuration,
            peer_configuration,
            peer_descriptor,
            peer_connection_state,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::manager::{ResourceManager, ResourceManagerRef};
    use opendut_types::peer::state::PeerConnectionState;
    use opendut_types::peer::PeerId;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::time::timeout;
    use crate::resource::persistence::resources::Persistable;

    #[tokio::test]
    async fn should_notify_about_resource_insertions() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();

        let mut subscription = resource_manager.subscribe().await;

        let id = PeerId::random();
        let timeout_duration = Duration::from_secs(10);

        let value = PeerConnectionState::Offline;
        resource_manager.insert(id, value.clone()).await?;
        assert_eq!(timeout(timeout_duration, subscription.receive()).await??, SubscriptionEvent::Inserted { id, value });

        let value = PeerConnectionState::Online { remote_host: IpAddr::from_str("127.0.0.1")? };
        resource_manager.insert(id, value.clone()).await?;
        assert_eq!(timeout(timeout_duration, subscription.receive()).await??, SubscriptionEvent::Inserted { id, value });

        Ok(())
    }

    #[tokio::test]
    async fn should_notify_about_resource_removal() -> anyhow::Result<()> {
        let fixture = SubscriptionFixture::new();
        let mut subscription = fixture.resource_manager.subscribe().await;

        let value = PeerConnectionState::Offline;
        fixture.resource_manager.insert(fixture.id, value.clone()).await?;
        assert_eq!(fixture.receive_notification(&mut subscription).await?, SubscriptionEvent::Inserted { id: fixture.id, value: value.clone() });

        fixture.resource_manager.remove::<PeerConnectionState>(fixture.id).await?;
        assert_eq!(fixture.receive_notification(&mut subscription).await?, SubscriptionEvent::Removed { id: fixture.id, value });

        Ok(())
    }

    #[tokio::test]
    async fn should_not_notify_if_nothing_was_removed() -> anyhow::Result<()> {
        let fixture = SubscriptionFixture::new();
        let mut subscription = fixture.resource_manager.subscribe::<PeerConnectionState>().await;

        fixture.resource_manager.remove::<PeerConnectionState>(fixture.id).await?;

        fixture.expect_no_notification(&mut subscription).await?;
        Ok(())
    }


    struct SubscriptionFixture {
        id: PeerId,
        resource_manager: ResourceManagerRef,
        timeout_duration: Duration,
    }

    impl SubscriptionFixture {
        pub fn new() -> Self {
            let resource_manager = ResourceManager::new_in_memory();
            let id = PeerId::random();
            let timeout_duration = Duration::from_secs(10);
            Self {
                id,
                resource_manager,
                timeout_duration,
            }
        }

        pub async fn receive_notification<R>(&self, subscription: &mut Subscription<R>) -> anyhow::Result<SubscriptionEvent<R>>
        where R: Resource + Persistable + Subscribable {
            Ok(timeout(self.timeout_duration, subscription.receive()).await??)
        }

        pub async fn expect_no_notification<R>(&self, subscription: &mut Subscription<R>) -> anyhow::Result<()>
        where R: Resource + Persistable + Subscribable {
            let result = timeout(Duration::from_secs(3), subscription.receive()).await;
            match result {
                Ok(result) => {
                    panic!("Received unexpected notification despite no notification was expected: {result:?}")
                }
                Err(_) => { Ok(()) }
            }
        }
    }
}
