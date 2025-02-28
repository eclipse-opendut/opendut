use std::net::IpAddr;
use serde::{Deserialize, Serialize};
use crate::cluster::ClusterId;
use crate::ShortName;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerState {
    /// Down can mean:
    /// - the peer was Up, then disconnected
    /// - the peer has no PeerState associated
    ///
    /// No PeerState happens when:
    /// - The peer has not been set up yet.
    /// - The peer has not been seen, since the last CARL restart.
    #[default]
    Down,
    Up {
        inner: PeerUpState,
        remote_host: IpAddr,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerUpState {
    Available,
    Blocked {
        inner: PeerBlockedState,
        by_cluster: ClusterId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PeerBlockedState {
    Deploying,
    Member,
    Undeploying,
}

impl ShortName for PeerState {
    fn short_name(&self) -> &'static str {
        match self {
            PeerState::Up { inner, .. } => match inner {
                PeerUpState::Available => "Available",
                PeerUpState::Blocked { inner: PeerBlockedState::Deploying, .. } => "Deploying",
                PeerUpState::Blocked { inner: PeerBlockedState::Member, .. } => "Member",
                PeerUpState::Blocked { inner: PeerBlockedState::Undeploying, .. } => "Undeploying",
            }
            PeerState::Down => "Down",
        }
    }
}
