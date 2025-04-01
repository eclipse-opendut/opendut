use crate::manager::cluster_manager;
use opendut_carl_api::carl::cluster::StoreClusterDeploymentError;

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
