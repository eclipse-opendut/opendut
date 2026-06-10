use std::collections::HashMap;
use std::net::IpAddr;
use serde::Serialize;
use opendut_model::cluster::ClusterId;
use opendut_model::peer::PeerId;
use opendut_model::util::Port;


/// Cluster assignments are determined in ClusterManager here [`crate::manager::cluster_manager::ClusterManager::rollout_cluster`]
/// The ports defined in the [`PeerClusterAssignment`] are used in the can manager in EDGAR opendut-edgar/src/service/can/can_manager.rs
/// CAN connection parameters are defined in the model [`opendut_model::peer::configuration::parameter::CanConnection`]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ClusterAssignment {
    pub id: ClusterId,
    pub leader: PeerId,
    pub assignments: HashMap<PeerId, PeerClusterAssignment>,
}
impl ClusterAssignment {
    pub fn leader_assignment(&self) -> Option<&PeerClusterAssignment> {
        self.assignments.get(&self.leader)
    }

    pub fn non_leader_assignments(&self) -> HashMap<PeerId, PeerClusterAssignment> {
        self.assignments_of_peers_directly_connected_to(self.leader)
    }

    /// Returns the `PeerClusterAssignment` entries for peers which are directly connected to the given peer.
    /// If the passed peer is the leader, these are all other peers.
    /// If it's any other peer, the leader is returned.
    pub fn assignments_of_peers_directly_connected_to(&self, local_peer_id: PeerId) -> HashMap<PeerId, PeerClusterAssignment> {
        let mut assignments = self.assignments.clone();
        assignments.remove(&local_peer_id);
        assignments
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PeerClusterAssignment {
    pub vpn_address: IpAddr,
    pub can_server_port: Port,
}
