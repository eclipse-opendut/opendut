use std::fmt::{Display, Formatter};
use serde::Serialize;
use opendut_model::peer::state::{PeerConnectionState, PeerState};

pub mod list;
pub mod describe;
pub mod create;
pub mod delete;
pub mod apply;

#[derive(Debug, PartialEq, Serialize)]
pub enum PeerStatus {
    Connected,
    Disconnected,
}

impl From<PeerState> for PeerStatus {
    fn from(peer_state: PeerState) -> Self {
        match peer_state.connection {
            PeerConnectionState::Offline => { PeerStatus::Disconnected }
            PeerConnectionState::Online { .. } => { PeerStatus::Connected }
        }
    }
}

impl Display for PeerStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerStatus::Connected => write!(f, "Connected"),
            PeerStatus::Disconnected => write!(f, "Disconnected"),
        }
    }
}
