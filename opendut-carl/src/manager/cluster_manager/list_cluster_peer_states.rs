use crate::manager::cluster_manager::{ListClusterPeersError, ListClusterPeersParams};
use crate::manager::cluster_manager;
use crate::resource::manager::ResourceManagerRef;
use opendut_types::cluster::ClusterId;
use opendut_types::peer::state::{PeerConnectionState, PeerMemberState, PeerState};
use opendut_types::peer::PeerId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::manager::peer_manager::list_peer_states::ListPeerStatesError;

#[derive(Clone)]
pub struct ListClusterPeerStatesParams {
    pub resource_manager: ResourceManagerRef,
    pub cluster_id: ClusterId,
}

pub async fn list_cluster_peer_states(params: ListClusterPeerStatesParams) -> Result<ClusterPeerStates, ListClusterPeerStatesError> {
    let ListClusterPeerStatesParams { resource_manager, cluster_id } = params;

    let cluster_peers = cluster_manager::list_cluster_peers(ListClusterPeersParams { cluster_id, resource_manager: Arc::clone(&resource_manager) }).await
        .map_err(|source| ListClusterPeerStatesError::ListClusterPeers { cluster_id, source })?
        .into_iter()
        .map(|peer| peer.id)
        .collect::<HashSet<_>>();
    let all_peer_states =
        resource_manager.resources(async |resources|
            resources.list_peer_states()
        ).await
            .map_err(|source| ListClusterPeerStatesError::ListPeerStates { cluster_id, source })?;

    let cluster_peer_states = all_peer_states
        .into_iter()
        .filter(|(peer_id, _)| cluster_peers.contains(peer_id))
        .collect::<HashMap<_, _>>();

    Ok(ClusterPeerStates::new(cluster_id, cluster_peer_states))
}

pub(crate) struct ClusterPeerStates {
    pub cluster_id: ClusterId,
    #[allow(unused)]
    peer_states: HashMap<PeerId, PeerState>,
    blocked_peers: HashMap<PeerId, ClusterId>,
    offline_peers: HashSet<PeerId>,
    blocked_peers_by_another_cluster: HashMap<PeerId, ClusterId>,
    only_blocked_by_self: bool,
}

impl ClusterPeerStates {
    pub fn new(cluster_id: ClusterId, peer_states: HashMap<PeerId, PeerState>) -> Self {
        let blocked_peers = peer_states
            .iter()
            .filter_map(|(peer_id, peer_state)| {
                if let PeerMemberState::Blocked { by_cluster } = peer_state.member {
                    Some((*peer_id, by_cluster))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();

        let blocked_peers_by_another_cluster = blocked_peers
            .clone()
            .into_iter()
            .filter(|(_, other_cluster_id)| *other_cluster_id != cluster_id)
            .collect::<HashMap<_, _>>();

        let only_blocked_by_self = blocked_peers.values().all(|other_cluster_id| other_cluster_id.eq(&cluster_id));

        let offline_peers = peer_states
            .iter()
            .filter_map(|(peer_id, peer_state)| {
                if let PeerConnectionState::Offline = peer_state.connection {
                    Some(peer_id)
                } else {
                    None
                }
            })
            .cloned()
            .collect::<HashSet<_>>();

        ClusterPeerStates {
            cluster_id,
            peer_states,
            blocked_peers,
            offline_peers,
            blocked_peers_by_another_cluster,
            only_blocked_by_self,
        }
    }

    pub fn check_cluster_deployable(&self) -> ClusterDeployable {
        let unavailable_peers = self.blocked_peers.keys().cloned().collect::<HashSet<_>>()
            .union(&self.offline_peers).cloned().collect::<HashSet<_>>();

        if unavailable_peers.is_empty() {
            ClusterDeployable::AllPeersAvailable
        }
        else if self.only_blocked_by_self && self.offline_peers.is_empty() {
            ClusterDeployable::AlreadyDeployed
        }
        else {
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers }
        }
    }

    pub fn check_all_peers_are_available_not_necessarily_online(&self) -> ClusterDeployable {
        if self.blocked_peers.is_empty() {
            ClusterDeployable::AllPeersAvailable
        }
        else if self.only_blocked_by_self {
            ClusterDeployable::AlreadyDeployed
        }
        else {
            let unavailable_peers = self.blocked_peers_by_another_cluster.clone()
                .into_keys()
                .collect::<HashSet<_>>();
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
pub enum ListClusterPeerStatesError {
    #[error("Listing the peer states for cluster <{cluster_id}> was not possible, because determining the peers of the cluster failed.")]
    ListClusterPeers { cluster_id: ClusterId, #[source] source: ListClusterPeersError },
    #[error("Failed to list peer states for cluster <{cluster_id}>.")]
    ListPeerStates { cluster_id: ClusterId, #[source] source: ListPeerStatesError },
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource::manager::ResourceManager;
    use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterName};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::peer::state::PeerConnectionState;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerNetworkDescriptor};
    use opendut_types::topology::DeviceName;
    use opendut_types::topology::{DeviceDescriptor, DeviceId, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use std::collections::HashSet;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[tokio::test]
    async fn should_filter_down_peers() -> anyhow::Result<()> {
        let Fixture { resource_manager, peer_a, peer_b, cluster, remote_host } = Fixture::create().await?;

        let params = ListClusterPeerStatesParams { resource_manager: resource_manager.clone(), cluster_id: cluster.id };

        let cluster_peer_states = list_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, PeerState { connection: PeerConnectionState::Offline, member: PeerMemberState::Available }),
            (peer_b.id, PeerState { connection: PeerConnectionState::Offline, member: PeerMemberState::Available }),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_a.id, peer_b.id]) }
        );

        let online_state = PeerConnectionState::Online { remote_host };
        let available_state = PeerState { connection: online_state.clone(), member: PeerMemberState::Available };

        resource_manager.insert(peer_a.id, online_state.clone()).await?;

        let cluster_peer_states = list_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, available_state.clone()),
            (peer_b.id, PeerState { connection: PeerConnectionState::Offline, member: PeerMemberState::Available }),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_b.id]) }
        );

        resource_manager.insert(peer_b.id, online_state.clone()).await?;

        let cluster_peer_states = list_cluster_peer_states(params.clone()).await?;
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
        // Given
        let Fixture { resource_manager, peer_a, peer_b, cluster, remote_host } = Fixture::create().await?;

        let params = ListClusterPeerStatesParams { resource_manager: resource_manager.clone(), cluster_id: cluster.id };
        let online_state = PeerConnectionState::Online { remote_host };
        let offline_state = PeerConnectionState::Offline;

        let other_cluster = ClusterConfiguration {
            id: ClusterId::random(),
            name: ClusterName::try_from("BlockingCluster")?,
            leader: cluster.leader,
            devices: cluster.devices.clone(),
        };
        // When another cluster is deployed
        {
            resource_manager.insert(other_cluster.id, other_cluster.clone()).await?;
            let other_cluster_deployment = ClusterDeployment { id: other_cluster.id };
            resource_manager.insert(peer_a.id, online_state.clone()).await?;
            resource_manager.insert(peer_b.id, online_state.clone()).await?;
            resource_manager.insert(other_cluster.id, other_cluster_deployment.clone()).await?;
        }

        let blocked_by_other_cluster_state = PeerState { connection: online_state.clone(), member: PeerMemberState::Blocked { by_cluster: other_cluster.id } };
        let blocked_by_own_cluster_state   = PeerState { connection: online_state.clone(), member: PeerMemberState::Blocked { by_cluster: cluster.id } };
        let available_state = PeerState { connection: PeerConnectionState::Online { remote_host}, member: PeerMemberState::Available };
        let available_but_offline_state = PeerState { connection: PeerConnectionState::Offline, member: PeerMemberState::Available };
        
        // Then the cluster peers are not available
        let cluster_peer_states = list_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, blocked_by_other_cluster_state.clone()),
            (peer_b.id, blocked_by_other_cluster_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_a.id, peer_b.id]) }
        );

        // When other deployment is removed
        resource_manager.remove::<ClusterDeployment>(other_cluster.id).await?;
        resource_manager.insert(peer_b.id, offline_state.clone()).await?;

        let cluster_peer_states = list_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, available_state.clone()),
            (peer_b.id, available_but_offline_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers: HashSet::from_iter(vec![peer_b.id]) }
        );

        resource_manager.insert(peer_b.id, online_state.clone()).await?;

        let cluster_peer_states = list_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, available_state.clone()),
            (peer_b.id, available_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::AllPeersAvailable
        );

        let cluster_deployment = ClusterDeployment { id: cluster.id };
        resource_manager.insert(cluster.id, cluster_deployment.clone()).await?;

        let cluster_peer_states = list_cluster_peer_states(params.clone()).await?;
        assert_eq!(cluster_peer_states.peer_states, HashMap::from_iter([
            (peer_a.id, blocked_by_own_cluster_state.clone()),
            (peer_b.id, blocked_by_own_cluster_state.clone()),
        ]));
        assert_eq!(
            cluster_peer_states.check_cluster_deployable(),
            ClusterDeployable::AlreadyDeployed
        );

        Ok(())
    }

    struct Fixture {
        resource_manager: ResourceManagerRef,
        peer_a: PeerDescriptor,
        peer_b: PeerDescriptor,
        cluster: ClusterConfiguration,
        remote_host: IpAddr,
    }
    impl Fixture {
        async fn create() -> anyhow::Result<Self> {
            let resource_manager = ResourceManager::new_in_memory();

            let peer_a = generate_peer_descriptor()?;
            resource_manager.insert(peer_a.id, peer_a.clone()).await?;

            let peer_b = generate_peer_descriptor()?;
            resource_manager.insert(peer_b.id, peer_b.clone()).await?;

            let peer_not_in_cluster = generate_peer_descriptor()?;
            resource_manager.insert(peer_not_in_cluster.id, peer_not_in_cluster.clone()).await?;

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
            resource_manager.insert(cluster.id, cluster.clone()).await?;

            Ok(Self {
                resource_manager,
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
