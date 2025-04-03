mod cluster_manager {
    use crate::manager::cluster_manager;
    use opendut_carl_api::carl::cluster::{CreateClusterConfigurationError, DeleteClusterConfigurationError, DeleteClusterDeploymentError, StoreClusterDeploymentError};

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

    impl From<cluster_manager::error::StoreClusterDeploymentError> for StoreClusterDeploymentError {
        fn from(value: cluster_manager::error::StoreClusterDeploymentError) -> Self {
            match value {
                cluster_manager::error::StoreClusterDeploymentError::IllegalPeerState { cluster_id, cluster_name, invalid_peers } =>
                    Self::IllegalPeerState { cluster_id, cluster_name, invalid_peers },
                cluster_manager::error::StoreClusterDeploymentError::ListClusterPeerStates { cluster_id, source: _ } => {
                    Self::Internal {
                        cluster_id,
                        cluster_name: None,
                        cause: String::from("Error when listing cluster peer states"),
                    }
                }
                cluster_manager::error::StoreClusterDeploymentError::Persistence { cluster_id, cluster_name, source: _ } => {
                    Self::Internal {
                        cluster_id,
                        cluster_name,
                        cause: String::from("Error when accessing persistence"),
                    }
                }
            }
        }
    }

    impl From<cluster_manager::DeleteClusterDeploymentError> for DeleteClusterDeploymentError {
        fn from(value: cluster_manager::DeleteClusterDeploymentError) -> Self {
            match value {
                cluster_manager::DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id } =>
                    Self::ClusterDeploymentNotFound { cluster_id },
                cluster_manager::DeleteClusterDeploymentError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } =>
                    Self::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states },
                cluster_manager::DeleteClusterDeploymentError::Persistence { cluster_id, cluster_name, source: _ } => {
                    Self::Internal {
                        cluster_id,
                        cluster_name,
                        cause: String::from("Error when accessing persistence while deleting cluster deployment"),
                    }
                }
                cluster_manager::DeleteClusterDeploymentError::VpnClient { cluster_id, cluster_name, source: _ } =>
                    Self::Internal {
                        cluster_id,
                        cluster_name: Some(cluster_name),
                        cause: String::from("Error when tearing down VPN while deleting cluster deployment"),
                    }
            }
        }
    }
}

mod peer_manager {
    use opendut_carl_api::carl::peer::StorePeerDescriptorError;
    use crate::manager::peer_manager;

    impl From<peer_manager::store_peer_descriptor::StorePeerDescriptorError> for StorePeerDescriptorError {
        fn from(value: peer_manager::store_peer_descriptor::StorePeerDescriptorError) -> Self {
            match value {
                peer_manager::store_peer_descriptor::StorePeerDescriptorError::IllegalPeerState { peer_id, peer_name, actual_state, required_states } =>
                    Self::IllegalPeerState { peer_id, peer_name, actual_state, required_states },
                peer_manager::store_peer_descriptor::StorePeerDescriptorError::Persistence { peer_id, peer_name, source: _ } =>
                    Self::Internal {
                        peer_id,
                        peer_name,
                        cause: String::from("Error when accessing persistence while storing peer descriptor"),
                    },
                peer_manager::store_peer_descriptor::StorePeerDescriptorError::VpnClient { peer_id, peer_name, source: _ } =>
                    Self::Internal {
                        peer_id,
                        peer_name,
                        cause: String::from("Error when creating peer in VPN management while storing peer descriptor"),
                    }
            }
        }
    }
}
