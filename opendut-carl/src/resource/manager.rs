pub use crate::resource::api::subscription::SubscriptionEvent;

use crate::resource::api::resources::{RelayedSubscriptionEvents, Resources};
use crate::resource::api::Resource;
use crate::resource::persistence::error::{MapErrToInner, PersistenceResult};
use crate::resource::persistence::persistable::Persistable;
use crate::resource::storage::{PersistenceOptions, ResourceStorage, ResourcesStorageApi};
use crate::resource::api::subscription::{self, ResourceSubscriptionChannels, Subscribable, Subscription};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use config::Config;
use tokio::sync::{RwLock, RwLockWriteGuard};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use crate::resource::ConnectError;


pub type ResourceManagerRef = Arc<ResourceManager>;

pub struct ResourceManager {
    state: RwLock<State>,
    event_listener_cancel: CancellationToken,
}

struct State {
    storage: ResourceStorage,
    subscribers: ResourceSubscriptionChannels,
}

impl ResourceManager {

    /// Create a ResourceManager from configured values.
    ///
    /// Returns a tuple with a reference for using the ResourceManager,
    /// and another reference which, when it goes out of scope, ensures
    /// the ResourceManager is shut down properly.
    pub async fn load_from_config(settings: &Config) -> Result<(ResourceManagerRef, ResourceManagerCancel), ConnectError> {
        let persistence_options = PersistenceOptions::load(settings)?;

        let resources = ResourceStorage::connect(&persistence_options).await?;
        let subscribers = ResourceSubscriptionChannels::default();

        let cancel = ResourceManagerCancel::new();
        let resource_manager = ResourceManagerRef::new(Self {
            state: RwLock::new(State { storage: resources, subscribers }),
            event_listener_cancel: cancel.token(),
        });

        Ok((resource_manager, cancel))
    }

    pub async fn insert<R>(&self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable {
        self.resources_mut(async |resources| {
            resources.insert(id, resource)
        }).await
        .map_err_to_inner(std::convert::identity)
    }

    pub async fn remove<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Subscribable {
        self.resources_mut(async |resources| {
            resources.remove(id)
        }).await
        .map_err_to_inner(std::convert::identity)
    }

    pub async fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        let state = self.state.read().await;
        state.storage.resources(async |resources| resources.get(id)).await?
    }

    pub async fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where R: Resource + Persistable + Clone {
        let state = self.state.read().await;
        state.storage.resources(async |resources| resources.list()).await?
    }

    pub async fn resources<F, T>(&self, closure: F) -> PersistenceResult<T>
    where
        F: AsyncFnOnce(&Resources) -> T,
    {
        let state = self.state.read().await;
        state.storage.resources(async move |transaction| {
            let transaction = Resources::new(transaction);
            closure(&transaction).await
        }).await
    }

    /// Allows grouping modifications to the database. This does multiple things:
    /// - Opens a database transaction and then either commits it, or rolls it back when you return an `Err` out of the closure.
    /// - Acquires the lock for the database mutex and keeps it until the end of the closure.
    /// - Groups the async calls, so we only have to await at the end.
    pub async fn resources_mut<F, T, E>(&self, closure: F) -> PersistenceResult<Result<T, E>>
    where
        F: AsyncFnOnce(&mut Resources) -> Result<T, E>,
        E: Display,
    {
        let mut state = self.state.write().await;
        let (result, relayed_subscription_events) = state.storage.resources_mut(async move |transaction| {
            let mut transaction = Resources::new(transaction);
            closure(&mut transaction).await
        }).await?;
        if result.is_ok() {
            Self::send_relayed_subscription_events(relayed_subscription_events, &mut state).await;
        }
        Ok(result)
    }


    /// Run a closure when a resource is inserted/updated or removed.
    ///
    /// This spawns a Tokio task, which may run in a separate thread.
    /// It will be automatically cancelled when the `ResourceManagerRef` is dropped.
    ///
    /// For more control, see: [ResourceManager::subscribe]
    pub async fn spawn_event_listener<R: Resource + Subscribable>(
        &self,
        on_event: impl AsyncFn(SubscriptionEvent<R>) + Send + 'static,
    ) {
        let resource_name = R::type_name();
        let mut subscription = self.subscribe::<R>().await;
        let cancel = self.event_listener_cancel.clone();

        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current().block_on(async move {
                loop {
                    tokio::select! {
                        received = subscription.receive() => match received {
                            Ok(event) => {
                                on_event(event).await;
                            }
                            Err(subscription::ReceiveError::Broadcast(error)) => match error {
                                tokio::sync::broadcast::error::RecvError::Closed => {
                                    warn!("ResourceManager subscription channel for {resource_name} closed. Aborting.");
                                    break;
                                }
                                tokio::sync::broadcast::error::RecvError::Lagged(skipped_messages) => {
                                    warn!("ResourceManager subscription channel for {resource_name} lagged behind and had to skip {skipped_messages} messages.");
                                },
                            }
                        },
                        _ = cancel.cancelled() => {
                            debug!("ResourceManager is cancelling subscription channel for {resource_name}.");
                            break;
                        }
                    }
                }
            })
        });
    }

    /// Get events when a resource is inserted/updated or removed.
    ///
    /// For an easier to use interface, see: [ResourceManager::spawn_event_listener]
    pub async fn subscribe<R>(&self) -> Subscription<R>
    where R: Resource + Subscribable {
        let mut state = self.state.write().await;
        state.subscribers.subscribe()
    }

    async fn send_relayed_subscription_events(
        relayed_subscription_events: RelayedSubscriptionEvents,
        state: &mut RwLockWriteGuard<'_, State>,
    ) {
        let ResourceSubscriptionChannels {
            mut cluster_descriptor,
            mut cluster_deployment,
            mut peer_configuration,
            mut peer_descriptor,
            mut peer_connection_state,
            mut peer_configuration_state,
            #[cfg(feature = "viper")] mut viper_source_descriptor,
            #[cfg(feature = "viper")] mut viper_test_run_descriptor,
            #[cfg(feature = "viper")] mut viper_run_deployment,
        } = relayed_subscription_events;

        macro_rules! send_relayed_subscription_events {
            ($field:expr) => {
                while let Ok(event) = $field.1.try_recv() {
                    state.subscribers
                        .notify(event)
                        .expect("should successfully send notification about event during resource transaction");
                }
            };
        }

        send_relayed_subscription_events!(cluster_descriptor);
        send_relayed_subscription_events!(cluster_deployment);
        send_relayed_subscription_events!(peer_configuration);
        send_relayed_subscription_events!(peer_descriptor);
        send_relayed_subscription_events!(peer_connection_state);
        send_relayed_subscription_events!(peer_configuration_state);

        #[cfg(feature = "viper")]
        {
            send_relayed_subscription_events!(viper_source_descriptor);
            send_relayed_subscription_events!(viper_test_run_descriptor);
            send_relayed_subscription_events!(viper_run_deployment);
        }
    }
}


/// Cancels the ResourceManager's event listeners when dropped.
///
/// This is a separate type from [`ResourceManagerRef`], because:
/// - Multiple references to the ResourceManagerRef will exist, as used by PeerManager, ClusterManager etc..
///   As such, it will never be dropped naturally.
/// - If we wrap the ResourceManagerRef with a struct that implements `Drop` and `Clone`,
///   then it will get dropped when any clone goes out of scope, which cancels prematurely.
pub struct ResourceManagerCancel {
    inner: CancellationToken,
}
impl ResourceManagerCancel {
    fn new() -> Self {
        Self { inner: CancellationToken::new() }
    }
    fn token(&self) -> CancellationToken {
        self.inner.clone()
    }
}
impl Drop for ResourceManagerCancel {
    fn drop(&mut self) {
        self.inner.cancel();
    }
}


#[cfg(test)]
impl ResourceManager {
    pub fn new_in_memory() -> (ResourceManagerRef, ResourceManagerCancel) {
        let resources = futures::executor::block_on(
            ResourceStorage::connect(&PersistenceOptions::Disabled)
        )
        .expect("Creating in-memory storage for tests should not fail");

        let subscribers = ResourceSubscriptionChannels::default();

        let cancel = ResourceManagerCancel::new();

        let resource_manager = ResourceManagerRef::new(Self {
            state: RwLock::new(State { storage: resources, subscribers }),
            event_listener_cancel: cancel.token(),
        });

        (resource_manager, cancel)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::time::Duration;
    use std::vec;

    use googletest::prelude::*;
    use tokio::sync::mpsc;

    use super::*;
    use opendut_model::cluster::{ClusterDeployment, ClusterDescriptor, ClusterId, ClusterName};
    use opendut_model::peer::executor::{container::{ContainerCommand, ContainerImage, ContainerName, Engine}, ExecutorDescriptor, ExecutorDescriptors, ExecutorId, ExecutorKind};
    use opendut_model::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_model::topology::Topology;
    use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    #[tokio::test]
    async fn should_support_create_read_update_delete_operations() -> Result<()> {

        let (testee, _cancel) = ResourceManager::new_in_memory();

        let peer_resource_id = PeerId::random();
        let peer = PeerDescriptor {
            id: peer_resource_id,
            name: PeerName::try_from("TestPeer")?,
            location: PeerLocation::try_from("Ulm").ok(),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: NetworkInterfaceId::random(),
                        name: NetworkInterfaceName::try_from("eth0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1")?),
            },
            topology: Topology::default(),
            executors: ExecutorDescriptors {
                executors: vec![
                    ExecutorDescriptor {
                        id: ExecutorId::random(),
                        kind: ExecutorKind::Container {
                            engine: Engine::Docker,
                            name: ContainerName::Empty,
                            image: ContainerImage::try_from("testUrl")?,
                            volumes: vec![],
                            devices: vec![],
                            envs: vec![],
                            ports: vec![],
                            command: ContainerCommand::Default,
                            args: vec![],
                        },
                        results_url: None,
                    }
                ],
            }
        };

        let cluster_resource_id = ClusterId::random();
        let cluster_descriptor = ClusterDescriptor {
            id: cluster_resource_id,
            name: ClusterName::try_from("ClusterX032")?,
            leader: peer.id,
            devices: HashSet::new(),
        };


        testee.insert(peer_resource_id, Clone::clone(&peer)).await?;
        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await?, some(eq(&peer)));

        testee.insert(cluster_resource_id, Clone::clone(&cluster_descriptor)).await?;
        assert_that!(testee.get::<ClusterDescriptor>(cluster_resource_id).await?, some(eq(&cluster_descriptor)));

        assert!(testee.list::<PeerDescriptor>().await?.contains_key(&peer_resource_id));

        assert_that!(testee.get::<PeerDescriptor>(PeerId::random()).await?, none());

        assert_that!(testee.remove::<PeerDescriptor>(peer_resource_id).await?, some(eq(&peer)));

        testee.insert(peer_resource_id, Clone::clone(&peer)).await?;

        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await?, some(eq(&peer)));

        testee.resources(async |resources| {
            resources.list::<ClusterDescriptor>()?
                .into_iter()
                .for_each(|(_cluster_id, cluster)| {
                    assert_that!(cluster, eq(&cluster_descriptor));
                });
            PersistenceResult::Ok(())
        }).await??;

        Ok(())
    }

    #[tokio::test]
    async fn should_spawn_an_event_listener() -> Result<()> {
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
}
