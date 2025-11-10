use serde::{Deserialize, Serialize};

use crate::ShortName;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClusterState {
    #[default]
    Undeployed,
    Deploying,
    Deployed(DeployedClusterState),
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

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum DeployedClusterState {
    #[default]
    Unhealthy,
    Healthy,
}
