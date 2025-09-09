use serde::{Deserialize, Serialize};

use crate::ShortName;

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

impl ShortName for ClusterState {
    fn short_name(&self) -> &'static str {
        match self {
            ClusterState::Undeployed => "Undeployed",
            ClusterState::Deploying => "Deploying",
            ClusterState::Deployed(inner) => match inner {
                DeployedClusterState::Unhealthy => "Unhealthy",
                DeployedClusterState::Healthy => "Healthy",
            }
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