use crate::resource::api::global::GlobalResourcesRef;
use crate::resource::api::resources::{RelayedSubscriptionEvents, Resources};
use crate::resource::api::Resource;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::resources::Persistable;
use crate::resource::storage::{PersistenceOptions, ResourceStorage, ResourcesStorageApi};
pub use crate::resource::subscription::SubscriptionEvent;
use crate::resource::subscription::{ResourceSubscriptionChannels, Subscribable, Subscription};
use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockWriteGuard};
use crate::resource::ConnectError;

pub type ResourceManagerRef = Arc<ResourceManager>;

pub struct ResourceManager {
    state: RwLock<State>,
    pub global: GlobalResourcesRef,
}

struct State {
    storage: ResourceStorage,
    subscribers: ResourceSubscriptionChannels,
}

impl ResourceManager {

    pub async fn create(global: GlobalResourcesRef, storage_options: PersistenceOptions) -> Result<ResourceManagerRef, ConnectError> {
        let resources = ResourceStorage::connect(storage_options).await?;
        let subscribers = ResourceSubscriptionChannels::default();

        Ok(Arc::new(Self {
            state: RwLock::new(State { storage: resources, subscribers }),
            global,
        }))
    }

    pub async fn insert<R>(&self, id: R::Id, resource: R) -> PersistenceResult<()>
    where R: Resource + Persistable + Subscribable {
        let mut state = self.state.write().await;

        let (result, relayed_subscription_events) = state.storage.resources_mut(self.global.clone(), async |transaction| {
            transaction.insert(id.clone(), resource.clone())
        }).await?;
        Self::send_relayed_subscription_events(relayed_subscription_events, &mut state).await;
        result
    }

    pub async fn remove<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Subscribable {
        let mut state = self.state.write().await;
        let (result, relayed_subscription_events) = state.storage.resources_mut(self.global.clone(), async move |transaction| {
            transaction.remove(id)
        }).await?;
        Self::send_relayed_subscription_events(relayed_subscription_events, &mut state).await;
        result
    }

    pub async fn get<R>(&self, id: R::Id) -> PersistenceResult<Option<R>>
    where R: Resource + Persistable + Clone {
        let state = self.state.read().await;
        state.storage.resources(self.global.clone(), async |resources| resources.get(id)).await?
    }

    pub async fn list<R>(&self) -> PersistenceResult<HashMap<R::Id, R>>
    where R: Resource + Persistable + Clone {
        let state = self.state.read().await;
        state.storage.resources(self.global.clone(), async |resources| resources.list()).await?
    }

    pub async fn resources<F, T>(&self, closure: F) -> PersistenceResult<T>
    where
        F: AsyncFnOnce(&Resources) -> T,
    {
        let state = self.state.read().await;
        state.storage.resources(self.global.clone(), async move |transaction| {
            closure(transaction).await
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
        let (result, relayed_subscription_events) = state.storage.resources_mut(self.global.clone(), async move |transaction| {
            closure(transaction).await
        }).await?;
        Self::send_relayed_subscription_events(relayed_subscription_events, &mut state).await;
        Ok(result)
    }

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
            mut cluster_configuration,
            mut cluster_deployment,
            mut old_peer_configuration,
            mut peer_configuration,
            mut peer_descriptor,
            mut peer_connection_state,
        } = relayed_subscription_events;


        while let Ok(event) = cluster_configuration.1.try_recv() {
            state.subscribers
                .notify(event)
                .expect("should successfully send notification about event during resource transaction");
        }

        while let Ok(event) = cluster_deployment.1.try_recv() {
            state.subscribers
                .notify(event)
                .expect("should successfully send notification about event during resource transaction");
        }

        while let Ok(event) = old_peer_configuration.1.try_recv() {
            state.subscribers
                .notify(event)
                .expect("should successfully send notification about event during resource transaction");
        }

        while let Ok(event) = peer_configuration.1.try_recv() {
            state.subscribers
                .notify(event)
                .expect("should successfully send notification about event during resource transaction");
        }

        while let Ok(event) = peer_descriptor.1.try_recv() {
            state.subscribers
                .notify(event)
                .expect("should successfully send notification about event during resource transaction");
        }

        while let Ok(event) = peer_connection_state.1.try_recv() {
            state.subscribers
                .notify(event)
                .expect("should successfully send notification about event during resource transaction");
        }
    }
}


#[cfg(test)]
impl ResourceManager {
    pub fn new_in_memory() -> ResourceManagerRef {
        let resources = futures::executor::block_on(
            ResourceStorage::connect(PersistenceOptions::Disabled)
        )
        .expect("Creating in-memory storage for tests should not fail");

        let subscribers = ResourceSubscriptionChannels::default();

        let mut global = crate::resource::api::global::GlobalResources::default();
        global.insert(crate::settings::vpn::Vpn::Disabled);
        let global = global.complete();

        Arc::new(Self {
            state: RwLock::new(State { storage: resources, subscribers }),
            global,
        })
    }

    async fn contains<R>(&self, id: R::Id) -> bool
    where R: Resource + Clone {
        let state = self.state.read().await;
        state.storage.contains::<R>(id).await
    }

    async fn is_empty(&self) -> bool {
        let state = self.state.read().await;
        state.storage.is_empty().await
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::ops::Not;
    use std::vec;

    use googletest::prelude::*;

    use super::*;
    use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
    use opendut_types::peer::executor::{container::{ContainerCommand, ContainerImage, ContainerName, Engine}, ExecutorDescriptor, ExecutorDescriptors, ExecutorId, ExecutorKind};
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::topology::Topology;
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    #[tokio::test]
    async fn test() -> Result<()> {

        let testee = ResourceManager::new_in_memory();

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
        let cluster_configuration = ClusterConfiguration {
            id: cluster_resource_id,
            name: ClusterName::try_from("ClusterX032")?,
            leader: peer.id,
            devices: HashSet::new(),
        };

        assert!(testee.is_empty().await);

        testee.insert(peer_resource_id, Clone::clone(&peer)).await?;

        assert!(testee.is_empty().await.not());

        testee.insert(cluster_resource_id, Clone::clone(&cluster_configuration)).await?;

        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await?, some(eq(&peer)));
        assert_that!(testee.get::<ClusterConfiguration>(cluster_resource_id).await?, some(eq(&cluster_configuration)));

        assert!(testee.contains::<PeerDescriptor>(peer_resource_id).await);

        assert_that!(testee.get::<PeerDescriptor>(PeerId::random()).await?, none());

        assert_that!(testee.remove::<PeerDescriptor>(peer_resource_id).await?, some(eq(&peer)));

        testee.insert(peer_resource_id, Clone::clone(&peer)).await?;

        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await?, some(eq(&peer)));

        testee.resources(async |resources| {
            resources.list::<ClusterConfiguration>()?
                .into_iter()
                .for_each(|(_cluster_id, cluster)| {
                    assert_that!(cluster, eq(&cluster_configuration));
                });
            PersistenceResult::Ok(())
        }).await??;

        Ok(())
    }
}
