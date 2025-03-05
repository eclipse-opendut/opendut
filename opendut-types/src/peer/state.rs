use std::net::IpAddr;
use serde::{Deserialize, Serialize};
use crate::cluster::ClusterId;
use crate::ShortName;


/// A peer state contains information about the connection state and the peer member state.
/// The PeerMemberState tells if the peer belongs to and is blocked by a cluster deployment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerState {
    pub connection: PeerConnectionState,
    pub member: PeerMemberState,
}

/// A peer may be either offline or online.
/// The PeerMessagingBroker is responsible for this information.
/// A peer connection state may not exist as a resource if the peer was not seen before.
#[derive(Clone, Default, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerConnectionState {
    #[default]
    Offline,
    Online {
        remote_host: IpAddr,
    },
}

/// The PeerMemberState tells if the peer is available or if it belongs to and is blocked by a cluster deployment.
/// A peer may be associated in multiple cluster configurations, but it may only be used in one cluster deployment.
/// The ClusterManager is responsible for this information.
/// The peer member state (peers' cluster membership) is derived from ClusterDeployment, ClusterConfiguration and PeerDescriptor resources.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerMemberState {
    Available,
    Blocked {
        by_cluster: ClusterId,
    }
}


impl Default for PeerState {
    fn default() -> Self {
        Self {
            connection: PeerConnectionState::Offline,
            member: PeerMemberState::Available,
        }
    }
}

impl ShortName for PeerState {
    fn short_name(&self) -> &'static str {
        match self.connection {
            PeerConnectionState::Offline => { "Offline" }
            PeerConnectionState::Online { .. } => {
                match self.member {
                    PeerMemberState::Available => { "Available" }
                    PeerMemberState::Blocked { .. } => { "Blocked" }
                }
            }
        }
    }
}
