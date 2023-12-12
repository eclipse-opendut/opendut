use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerStates(pub Vec<PeerState>);

impl Default for PeerState {
    fn default() -> Self {
        Self::Down
    }
}

impl Display for PeerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerState::Up(PeerUpState::Available) => write!(f, "Available"),
            PeerState::Up(PeerUpState::Blocked(PeerBlockedState::Deploying)) => write!(f, "Deploying"),
            PeerState::Up(PeerUpState::Blocked(PeerBlockedState::Member)) => write!(f, "Member"),
            PeerState::Up(PeerUpState::Blocked(PeerBlockedState::Undeploying)) => write!(f, "Undeploying"),
            PeerState::Down => write!(f, "Down"),
        }
    }
}

impl Display for PeerStates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let states = self.0.iter()
            .map(|state| state.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{states}")
    }
}
