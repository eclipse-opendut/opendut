use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClusterState {
    Undeployed,
    Deploying,
    Deployed(DeployedClusterState),
}

impl Default for ClusterState {
    fn default() -> Self {
        Self::Undeployed
    }
}

impl Display for ClusterState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClusterState::Undeployed => write!(f, "Undeployed"),
            ClusterState::Deploying => write!(f, "Deploying"),
            ClusterState::Deployed(DeployedClusterState::Unhealthy) => write!(f, "Unhealthy"),
            ClusterState::Deployed(DeployedClusterState::Healthy) => write!(f, "Healthy"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DeployedClusterState {
    Unhealthy,
    Healthy,
}

impl Default for DeployedClusterState {
    fn default() -> Self {
        Self::Unhealthy
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClusterStates(pub Vec<ClusterState>);

impl Display for ClusterStates {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let states = self.0.iter()
            .map(|state| state.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{states}")
    }
}
