use crate::actions;
use crate::actions::{DetermineClusterPeersError, DetermineClusterPeersParams};
use crate::persistence::error::{FlattenPersistenceResult, PersistenceError};
use crate::resources::manager::ResourcesManagerRef;
use crate::resources::storage::ResourcesStorageApi;
use opendut_types::cluster::ClusterId;
use opendut_types::peer::state::{PeerState, PeerUpState};
use opendut_types::peer::PeerId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
pub struct DetermineClusterPeerStatesParams {
    pub resources_manager: ResourcesManagerRef,
    pub cluster_id: ClusterId,
}

pub async fn determine_cluster_peer_states(params: DetermineClusterPeerStatesParams) -> Result<ClusterPeerStates, DetermineClusterPeerStatesError> {
    let DetermineClusterPeerStatesParams { resources_manager, cluster_id } = params;

    let cluster_peers = actions::determine_cluster_peers(DetermineClusterPeersParams { cluster_id, resources_manager: Arc::clone(&resources_manager) }).await
        .map_err(|source| DetermineClusterPeerStatesError::DetermineClusterPeers { cluster_id, source })?;

    let cluster_peer_states = resources_manager.resources_mut(|resources| {
        let cluster_peer_states = cluster_peers.into_iter().map(|peer| {
            let peer_state = resources.get::<PeerState>(peer.id)?.unwrap_or_default();

            Ok::<_, PersistenceError>((peer.id, peer_state))
        }).collect::<Result<HashMap<_, _>, _>>()?;

        Ok::<_, PersistenceError>(cluster_peer_states)
    }).await
        .flatten_persistence_result()
        .map_err(|source| DetermineClusterPeerStatesError::Persistence { cluster_id, source })?;

    Ok(ClusterPeerStates { cluster_id, peer_states: cluster_peer_states })
}

pub(crate) struct ClusterPeerStates {
    pub cluster_id: ClusterId,
    pub peer_states: HashMap<PeerId, PeerState>,
}
impl ClusterPeerStates {

    pub fn check_cluster_deployable(&self) -> ClusterDeployable {
        let mut only_blocked_by_self = true;

        let unavailable_peers: HashSet<PeerId> = self.peer_states
            .iter()
            .filter_map(|(peer_id, peer_state)| {
                if let PeerState::Down = peer_state {
                    only_blocked_by_self = false;
                    Some(peer_id)
                } else if let PeerState::Up { inner: PeerUpState::Blocked { by_cluster, .. }, .. } = peer_state {
                    if by_cluster != &self.cluster_id {
                        only_blocked_by_self = false;
                    }
                    Some(peer_id)
                } else {
                    None
                }
            })
            .cloned()
            .collect();

        if unavailable_peers.is_empty() {
            ClusterDeployable::AllPeersAvailable
        }
        else if only_blocked_by_self {
            ClusterDeployable::AlreadyDeployed
        }
        else {
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ClusterDeployable {
    AllPeersAvailable,
    AlreadyDeployed,
    NotAllPeersAvailable { unavailable_peers: HashSet<PeerId> },
}

#[derive(thiserror::Error, Debug)]
pub enum DetermineClusterPeerStatesError {
    #[error("Determining the peer states for cluster <{cluster_id}> was not possible, because determining the cluster peers failed.")]
    DetermineClusterPeers { cluster_id: ClusterId, #[source] source: DetermineClusterPeersError },
    #[error("Error while accessing persistence for determining peer states of cluster <{cluster_id}>.")]
    Persistence { cluster_id: ClusterId, #[source] source: PersistenceError },
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::manager::ResourcesManager;
    use opendut_types::cluster::{ClusterConfiguration, ClusterName};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::peer::state::{PeerBlockedState, PeerUpState};
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerNetworkDescriptor};
    use opendut_types::topology::DeviceName;
    use opendut_types::topology::{DeviceDescriptor, DeviceId, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use std::collections::HashSet;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[tokio::test]
    async fn should_filter_down_peers() -> anyhow::Result<()> {
        let Fixture { resources_manager, peer_a, peer_b, cluster, remote_host } = Fixture::create().await?;

        let params = DetermineClusterPeerStatesParams { resources_manager: resources_manager.clone(), cluster_id: cluster.id };

        let cluster_peer_states = actions::determine_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, PeerState::Down),
            (peer_b.id, PeerState::Down),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_a.id, peer_b.id]) }
        );


        let available_state = PeerState::Up { inner: PeerUpState::Available, remote_host };

        resources_manager.insert(peer_a.id, available_state.clone()).await?;

        let cluster_peer_states = actions::determine_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, available_state.clone()),
            (peer_b.id, PeerState::Down),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_b.id]) }
        );

        resources_manager.insert(peer_b.id, available_state.clone()).await?;

        let cluster_peer_states = actions::determine_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, available_state.clone()),
            (peer_b.id, available_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::AllPeersAvailable
        );

        Ok(())
    }

    #[tokio::test]
    async fn should_filter_blocked_peers() -> anyhow::Result<()> {
        let Fixture { resources_manager, peer_a, peer_b, cluster, remote_host } = Fixture::create().await?;

        let params = DetermineClusterPeerStatesParams { resources_manager: resources_manager.clone(), cluster_id: cluster.id };

        let blocked_by_other_cluster_state = PeerState::Up { inner: PeerUpState::Blocked { by_cluster: ClusterId::random(), inner: PeerBlockedState::Member }, remote_host };
        let blocked_by_own_cluster_state   = PeerState::Up { inner: PeerUpState::Blocked { by_cluster: cluster.id, inner: PeerBlockedState::Member }, remote_host };
        let available_state = PeerState::Up { inner: PeerUpState::Available, remote_host };

        resources_manager.insert(peer_a.id, blocked_by_other_cluster_state.clone()).await?;
        resources_manager.insert(peer_b.id, blocked_by_other_cluster_state.clone()).await?;

        let cluster_peer_states = actions::determine_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, blocked_by_other_cluster_state.clone()),
            (peer_b.id, blocked_by_other_cluster_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_a.id, peer_b.id]) }
        );

        resources_manager.insert(peer_a.id, available_state.clone()).await?;

        let cluster_peer_states = actions::determine_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, available_state.clone()),
            (peer_b.id, blocked_by_other_cluster_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_b.id]) }
        );

        resources_manager.insert(peer_b.id, available_state.clone()).await?;

        let cluster_peer_states = actions::determine_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, available_state.clone()),
            (peer_b.id, available_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::AllPeersAvailable
        );


        resources_manager.insert(peer_a.id, blocked_by_own_cluster_state.clone()).await?;

        let cluster_peer_states = actions::determine_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, blocked_by_own_cluster_state.clone()),
            (peer_b.id, available_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::AlreadyDeployed
        );

        Ok(())
    }

    struct Fixture {
        resources_manager: ResourcesManagerRef,
        peer_a: PeerDescriptor,
        peer_b: PeerDescriptor,
        cluster: ClusterConfiguration,
        remote_host: IpAddr,
    }
    impl Fixture {
        async fn create() -> anyhow::Result<Self> {
            let resources_manager = ResourcesManager::new_in_memory();

            let peer_a = generate_peer_descriptor()?;
            resources_manager.insert(peer_a.id, peer_a.clone()).await?;

            let peer_b = generate_peer_descriptor()?;
            resources_manager.insert(peer_b.id, peer_b.clone()).await?;

            let peer_not_in_cluster = generate_peer_descriptor()?;
            resources_manager.insert(peer_not_in_cluster.id, peer_not_in_cluster.clone()).await?;

            let cluster = ClusterConfiguration {
                id: ClusterId::random(),
                name: ClusterName::try_from("cluster")?,
                leader: peer_a.id,
                devices: HashSet::from_iter(
                    peer_a.topology.devices.iter()
                        .chain(peer_b.topology.devices.iter())
                        .map(|device| device.id)
                ),
            };
            resources_manager.insert(cluster.id, cluster.clone()).await?;

            Ok(Self {
                resources_manager,
                peer_a,
                peer_b,
                cluster,
                remote_host: IpAddr::from_str("127.0.0.1")?, //doesn't matter
            })
        }
    }

    fn generate_peer_descriptor() -> anyhow::Result<PeerDescriptor> {
        let network_interface_id = NetworkInterfaceId::random();

        Ok(PeerDescriptor {
            id: PeerId::random(),
            name: PeerName::try_from("peer")?,
            location: None,
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: network_interface_id,
                        name: NetworkInterfaceName::try_from("eth0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                ],
                bridge_name: None,
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: DeviceId::random(),
                        name: DeviceName::try_from("device")?,
                        description: None,
                        interface: network_interface_id,
                        tags: vec![],
                    }
                ],
            },
            executors: ExecutorDescriptors {
                executors: vec![],
            },
        })
    }
}
