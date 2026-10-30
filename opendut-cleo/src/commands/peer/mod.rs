use std::fmt::{Display, Formatter};
use serde::Serialize;
use opendut_model::cluster::ClusterId;
use opendut_model::peer::state::{PeerConnectionState, PeerMemberState, PeerState};

pub mod list;
pub mod describe;
pub mod create;
pub mod delete;
pub mod apply;


#[derive(Debug, PartialEq, Serialize)]
enum PeerStatus {
    Offline,
    Available,
    Blocked(ClusterId),
}

impl From<PeerState> for PeerStatus {
    fn from(peer_state: PeerState) -> Self {
        match peer_state.connection {
            PeerConnectionState::Offline => Self::Offline,
            PeerConnectionState::Online { .. } => match peer_state.member {
                PeerMemberState::Available => Self::Available,
                PeerMemberState::Blocked { by_cluster } => Self::Blocked(by_cluster),
            },
        }
    }
}

impl Display for PeerStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blocked(cluster) => write!(f, "Blocked by {cluster}"),
            Self::Offline => write!(f, "Offline"),
            Self::Available => write!(f, "Available"),
        }
    }
}
