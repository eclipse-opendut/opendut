use serde::{Deserialize, Serialize};
use crate::ShortName;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerState {
    Down,
    Up(PeerUpState),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerUpState {
    Available,
    Blocked(PeerBlockedState),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerBlockedState {
    Deploying,
    Member,
    Undeploying,
}

impl Default for PeerState {
    fn default() -> Self {
        Self::Down
    }
}

impl ShortName for PeerState {
    fn short_name(&self) -> &'static str {
        match self {
            PeerState::Up(inner) => match inner {
                PeerUpState::Available => "Available",
                PeerUpState::Blocked(PeerBlockedState::Deploying) => "Deploying",
                PeerUpState::Blocked(PeerBlockedState::Member) => "Member",
                PeerUpState::Blocked(PeerBlockedState::Undeploying) => "Undeploying",
            }
            PeerState::Down => "Down",
        }
    }
}
