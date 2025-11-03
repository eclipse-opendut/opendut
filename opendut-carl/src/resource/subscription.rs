use crate::resource::api::Resource;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment};
use opendut_model::peer::configuration::{OldPeerConfiguration, PeerConfiguration, EdgePeerConfigurationState};
use opendut_model::peer::state::PeerConnectionState;
use opendut_model::peer::PeerDescriptor;
use tokio::sync::broadcast;

#[cfg(feature = "viper")]
use opendut_model::viper::TestSuiteSourceDescriptor;

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
        impl $crate::resource::subscription::Subscribable for $resource {
            fn resource_subscribers_field(
                resource_subscribers: &mut $crate::resource::subscription::ResourceSubscriptionChannels
            ) -> &tokio::sync::broadcast::Sender<$crate::resource::subscription::SubscriptionEvent<Self>>
            where Self: Sized {
                let (sender, _) = &resource_subscribers.$field;
                sender
            }
        }
    }
}
impl_subscribable!(ClusterDeployment, cluster_deployment);
impl_subscribable!(ClusterDescriptor, cluster_descriptor);
impl_subscribable!(OldPeerConfiguration, old_peer_configuration);
impl_subscribable!(PeerConfiguration, peer_configuration);
impl_subscribable!(PeerDescriptor, peer_descriptor);
impl_subscribable!(PeerConnectionState, peer_connection_state);
impl_subscribable!(EdgePeerConfigurationState, peer_configuration_state);
#[cfg(feature = "viper")]
impl_subscribable!(TestSuiteSourceDescriptor, test_suite_source_descriptor);

mod deprecated {
    #![expect(deprecated)]
    impl_subscribable!(crate::startup::migration::ClusterConfiguration, _cluster_configuration); //TODO remove
}

pub type ResourceSubscriptionChannel<R> = (broadcast::Sender<SubscriptionEvent<R>>, broadcast::Receiver<SubscriptionEvent<R>>); //store both the sender and initial receiver, to prevent channel from closing

pub struct ResourceSubscriptionChannels {
    pub cluster_deployment: ResourceSubscriptionChannel<ClusterDeployment>,
    pub cluster_descriptor: ResourceSubscriptionChannel<ClusterDescriptor>,
    pub old_peer_configuration: ResourceSubscriptionChannel<OldPeerConfiguration>,
    pub peer_configuration: ResourceSubscriptionChannel<PeerConfiguration>,
    pub peer_descriptor: ResourceSubscriptionChannel<PeerDescriptor>,
    pub peer_connection_state: ResourceSubscriptionChannel<PeerConnectionState>,
    pub peer_configuration_state: ResourceSubscriptionChannel<EdgePeerConfigurationState>,
    #[cfg(feature = "viper")]
    pub test_suite_source_descriptor: ResourceSubscriptionChannel<TestSuiteSourceDescriptor>,

    #[deprecated] #[expect(deprecated)]
    pub(crate) _cluster_configuration: ResourceSubscriptionChannel<crate::startup::migration::ClusterConfiguration>, //TODO remove (should be unused)
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
        let ResourceSubscriptionChannels {
            cluster_deployment,
            cluster_descriptor,
            old_peer_configuration,
            peer_configuration,
            peer_descriptor,
            peer_connection_state,
            peer_configuration_state,
            #[cfg(feature = "viper")]
            test_suite_source_descriptor,
            #[expect(deprecated)]
            _cluster_configuration,
        } = self;

        let result =
            cluster_deployment.0.is_empty()
            && cluster_descriptor.0.is_empty()
            && old_peer_configuration.0.is_empty()
            && peer_configuration.0.is_empty()
            && peer_descriptor.0.is_empty()
            && peer_connection_state.0.is_empty()
            && peer_configuration_state.0.is_empty();

        #[cfg(feature = "viper")]
        let result = result && test_suite_source_descriptor.0.is_empty();

        result
    }
}
impl Default for ResourceSubscriptionChannels {
    fn default() -> Self {
        let capacity = 100;

        Self {
            cluster_deployment: broadcast::channel(capacity),
            cluster_descriptor: broadcast::channel(capacity),
            old_peer_configuration: broadcast::channel(capacity),
            peer_configuration: broadcast::channel(capacity),
            peer_descriptor: broadcast::channel(capacity),
            peer_connection_state: broadcast::channel(capacity),
            peer_configuration_state: broadcast::channel(capacity),
            #[cfg(feature = "viper")]
            test_suite_source_descriptor: broadcast::channel(capacity),
            #[expect(deprecated)]
            _cluster_configuration: broadcast::channel(1),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::peer_manager::tests::create_peer_descriptor;
    use crate::resource::manager::{ResourceManager, ResourceManagerRef};
    use crate::resource::persistence::persistable::Persistable;
    use crate::resource::storage::ResourcesStorageApi;
    use opendut_model::peer::state::PeerConnectionState;
    use opendut_model::peer::PeerId;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::time::Duration;
    use tokio::time::timeout;

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
    async fn should_notify_changes_in_transaction() -> anyhow::Result<()> {
        let fixture = SubscriptionFixture::new();
        let mut subscription = fixture.resource_manager.subscribe::<PeerConnectionState>().await;

        let value = PeerConnectionState::Offline;
        let _ = fixture.resource_manager.resources_mut(async |resources| {
            resources.insert(fixture.id, value.clone())
        }).await;

        assert_eq!(fixture.receive_notification(&mut subscription).await?, SubscriptionEvent::Inserted { id: fixture.id, value: value.clone() });
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

        let result = fixture.resource_manager.remove::<PeerConnectionState>(fixture.id).await?;
        assert!(result.is_none(), "Expected no peer connection state present!");

        SubscriptionFixture::expect_no_notification(&mut subscription).await?;
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn should_not_notify_if_transaction_was_aborted() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let id = PeerId::random();
        let peer_descriptor = create_peer_descriptor(id);
        let mut subscription = resource_manager.subscribe::<PeerDescriptor>().await;
        assert!(resource_manager.get::<PeerDescriptor>(id).await?.is_none(), "Expected no connection state present!");

        let transaction_result = resource_manager.resources_mut::<_, PeerDescriptor, anyhow::Error>(async |resources| {
            let value = peer_descriptor.clone();
            let _ = resources.insert(id, value.clone());
            Err(anyhow::anyhow!("Abort transaction"))
        }).await;
        
        assert!(matches!(transaction_result, Ok(Err(_))), "Expected inner transaction result to be erroneous.");
        assert!(resource_manager.get::<PeerDescriptor>(id).await?.is_none(), "Expected no connection state present!");
        SubscriptionFixture::expect_no_notification(&mut subscription).await?;
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

        pub async fn expect_no_notification<R>(subscription: &mut Subscription<R>) -> anyhow::Result<()>
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
