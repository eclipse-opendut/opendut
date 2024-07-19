use std::any::Any;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::resources::{IntoId, Resources};

pub type ResourcesManagerRef = Arc<ResourcesManager>;

pub struct ResourcesManager {
    state: RwLock<State>,
}

struct State {
    resources: Resources
}

impl ResourcesManager {

    pub fn new() -> ResourcesManagerRef {
        Arc::new(Self {
            state: RwLock::new(State {
                resources: Default::default()
            })
        })
    }

    pub async fn insert<R>(&self, id: impl IntoId<R>, resource: R)
    where R: Any  + Send + Sync {
        let mut state = self.state.write().await;
        state.resources.insert(id, resource);
    }

    pub async fn remove<R>(&self, id: impl IntoId<R>) -> Option<R>
    where R: Any  + Send + Sync {
        let mut state = self.state.write().await;
        state.resources.remove(id)
    }

    pub async fn get<R>(&self, id: impl IntoId<R>) -> Option<R>
    where R: Any  + Send + Sync + Clone {
        let state = self.state.read().await;
        state.resources.get(id)
    }

    pub async fn resources<F, T>(&self, f: F) -> T
    where F: FnOnce(&Resources) -> T {
        let state = self.state.read().await;
        f(&state.resources)
    }

    pub async fn resources_mut<F, T>(&self, f: F) -> T
    where F: FnOnce(&mut Resources) -> T {
        let mut state = self.state.write().await;
        f(&mut state.resources)
    }
}

#[cfg(test)]
impl ResourcesManager {
    async fn contains<R>(&self, id: impl IntoId<R>) -> bool
    where R: Any + Send + Sync + Clone {
        let state = self.state.read().await;
        state.resources.contains(id)
    }

    async fn is_empty(&self) -> bool {
        let state = self.state.read().await;
        state.resources.is_empty()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::ops::Not;
    use std::vec;

    use googletest::prelude::*;

    use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::peer::executor::{container::{ContainerCommand, ContainerImage, ContainerName, Engine}, ExecutorKind, ExecutorDescriptors, ExecutorDescriptor};
    use opendut_types::topology::Topology;
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

    use super::*;

    #[tokio::test]
    async fn test() -> Result<()> {

        let testee = ResourcesManager::new();

        let peer_resource_id = PeerId::random();
        let peer = PeerDescriptor {
            id: peer_resource_id,
            name: PeerName::try_from("TestPeer").unwrap(),
            location: PeerLocation::try_from("Ulm").ok(),
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        name: NetworkInterfaceName::try_from("eth0").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
            },
            topology: Topology::default(),
            executors: ExecutorDescriptors {
                executors: vec![
                    ExecutorDescriptor{
                        kind: ExecutorKind::Container {
                            engine: Engine::Docker,
                            name: ContainerName::Empty,
                            image: ContainerImage::try_from("testUrl").unwrap(),
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
            name: ClusterName::try_from("ClusterX032").unwrap(),
            leader: peer.id,
            devices: HashSet::new(),
        };

        assert!(testee.is_empty().await);

        testee.insert(peer_resource_id, Clone::clone(&peer)).await;

        assert!(testee.is_empty().await.not());

        testee.insert(cluster_resource_id, Clone::clone(&cluster_configuration)).await;

        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await, some(eq(Clone::clone(&peer))));
        assert_that!(testee.get::<ClusterConfiguration>(cluster_resource_id).await, some(eq(Clone::clone(&cluster_configuration))));

        assert!(testee.contains::<PeerDescriptor>(peer_resource_id).await);

        assert_that!(testee.get::<PeerDescriptor>(PeerId::random()).await, none());

        assert_that!(testee.remove::<PeerDescriptor>(peer_resource_id).await, some(eq(Clone::clone(&peer))));

        let id = testee.resources_mut(|resources| {
            resources.insert(peer_resource_id, Clone::clone(&peer));
            peer_resource_id
        }).await;

        assert_that!(testee.get::<PeerDescriptor>(id).await, some(eq(Clone::clone(&peer))));

        testee.resources(|resources| {
            resources.iter::<ClusterConfiguration>()
                .for_each(|cluster| {
                    assert_that!(Clone::clone(cluster), eq(Clone::clone(&cluster_configuration)));
                });
        }).await;

        testee.resources_mut(|resources| {
            resources.iter_mut::<PeerDescriptor>()
                .for_each(|peer| {
                    peer.name = PeerName::try_from("ChangedPeer").unwrap()
                });
        }).await;

        assert_that!(testee.get::<PeerDescriptor>(peer_resource_id).await, some(not(eq(Clone::clone(&peer)))));

        Ok(())
    }
}
