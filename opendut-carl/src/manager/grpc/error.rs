use crate::manager::cluster_manager;
use opendut_carl_api::carl::cluster::{CreateClusterConfigurationError, DeleteClusterConfigurationError, StoreClusterDeploymentError};

impl From<cluster_manager::StoreClusterDeploymentError> for StoreClusterDeploymentError {
    fn from(value: cluster_manager::StoreClusterDeploymentError) -> Self {
        match value {
            cluster_manager::StoreClusterDeploymentError::IllegalPeerState { cluster_id, cluster_name, invalid_peers } =>
                Self::IllegalPeerState { cluster_id, cluster_name, invalid_peers },
            cluster_manager::StoreClusterDeploymentError::ListClusterPeerStates { cluster_id, source: _ } => {
                Self::Internal {
                    cluster_id,
                    cluster_name: None,
                    cause: String::from("Error when listing cluster peer states"),
                }
            }
            cluster_manager::StoreClusterDeploymentError::Persistence { cluster_id, cluster_name, source: _ } => {
                Self::Internal {
                    cluster_id,
                    cluster_name,
                    cause: String::from("Error when accessing persistence"),
                }
            }
        }
    }
}

impl From<cluster_manager::CreateClusterConfigurationError> for CreateClusterConfigurationError {
    fn from(value: cluster_manager::CreateClusterConfigurationError) -> Self {
        match value {
            cluster_manager::CreateClusterConfigurationError::Persistence { cluster_id, cluster_name, source: _ } => {
                Self::Internal {
                    cluster_id,
                    cluster_name,
                    cause: String::from("Error when accessing persistence"),
                }
            }
        }
    }
}

impl From<cluster_manager::DeleteClusterConfigurationError> for DeleteClusterConfigurationError {
    fn from(value: cluster_manager::DeleteClusterConfigurationError) -> Self {
        match value {
            cluster_manager::DeleteClusterConfigurationError::ClusterDeploymentFound { cluster_id } =>
                Self::ClusterDeploymentFound { cluster_id },
            cluster_manager::DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id } =>
                Self::ClusterConfigurationNotFound { cluster_id },
            cluster_manager::DeleteClusterConfigurationError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } =>
                Self::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states },
            cluster_manager::DeleteClusterConfigurationError::Persistence { cluster_id, cluster_name, source: _ } => {
                Self::Internal {
                    cluster_id,
                    cluster_name,
                    cause: String::from("Error when accessing persistence"),
                }
            }
        }
    }
}
