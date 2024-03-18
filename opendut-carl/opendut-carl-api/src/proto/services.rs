pub mod cluster_manager {
    use opendut_types::cluster::{ClusterId, ClusterName};
    use opendut_types::cluster::state::ClusterState;
    use opendut_types::proto;
    use opendut_types::proto::{ConversionError, ConversionErrorBuilder};

    use crate::carl::cluster::{CreateClusterConfigurationError, DeleteClusterConfigurationError, DeleteClusterDeploymentError, StoreClusterDeploymentError};

    tonic::include_proto!("opendut.carl.services.cluster_manager");

    impl From<CreateClusterConfigurationError> for CreateClusterConfigurationFailure {
        fn from(error: CreateClusterConfigurationError) -> Self {
            let proto_error = match error {
                CreateClusterConfigurationError::ClusterConfigurationAlreadyExists { actual_id, actual_name, other_id, other_name } => {
                    create_cluster_configuration_failure::Error::ClusterConfigurationAlreadyExists(CreateClusterConfigurationFailureClusterConfigurationAlreadyExists {
                        actual_id: Some(actual_id.into()),
                        actual_name: Some(actual_name.into()),
                        other_id: Some(other_id.into()),
                        other_name: Some(other_name.into()),
                    })
                }
                CreateClusterConfigurationError::Internal { cluster_id, cluster_name, cause } => {
                    create_cluster_configuration_failure::Error::Internal(CreateClusterConfigurationFailureInternal {
                        cluster_id: Some(cluster_id.into()),
                        cluster_name: Some(cluster_name.into()),
                        cause
                    })
                }
            };
            CreateClusterConfigurationFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<CreateClusterConfigurationFailure> for CreateClusterConfigurationError {
        type Error = ConversionError;
        fn try_from(failure: CreateClusterConfigurationFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<CreateClusterConfigurationFailure, CreateClusterConfigurationError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                create_cluster_configuration_failure::Error::ClusterConfigurationAlreadyExists(error) => {
                    error.try_into()?
                }
                create_cluster_configuration_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<CreateClusterConfigurationFailureClusterConfigurationAlreadyExists> for CreateClusterConfigurationError {
        type Error = ConversionError;
        fn try_from(failure: CreateClusterConfigurationFailureClusterConfigurationAlreadyExists) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<CreateClusterConfigurationFailureClusterConfigurationAlreadyExists, CreateClusterConfigurationError>;
            let actual_id: ClusterId = failure.actual_id
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_id' not set"))?
                .try_into()?;
            let actual_name: ClusterName = failure.actual_name
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_name' not set"))?
                .try_into()?;
            let other_id: ClusterId = failure.other_id
                .ok_or_else(|| ErrorBuilder::new("Field 'other_id' not set"))?
                .try_into()?;
            let other_name: ClusterName = failure.other_name
                .ok_or_else(|| ErrorBuilder::new("Field 'other_name' not set"))?
                .try_into()?;
            Ok(CreateClusterConfigurationError::ClusterConfigurationAlreadyExists { actual_id, actual_name, other_id, other_name })
        }
    }

    impl TryFrom<CreateClusterConfigurationFailureInternal> for CreateClusterConfigurationError {
        type Error = ConversionError;
        fn try_from(failure: CreateClusterConfigurationFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<CreateClusterConfigurationFailureInternal, CreateClusterConfigurationError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            let cluster_name: ClusterName = failure.cluster_name
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_name' not set"))?
                .try_into()?;
            Ok(CreateClusterConfigurationError::Internal { cluster_id, cluster_name, cause: failure.cause })
        }
    }

    impl From<DeleteClusterConfigurationError> for DeleteClusterConfigurationFailure {
        fn from(error: DeleteClusterConfigurationError) -> Self {
            let proto_error = match error {
                DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id } => {
                    delete_cluster_configuration_failure::Error::ClusterConfigurationNotFound(DeleteClusterConfigurationFailureClusterConfigurationNotFound {
                        cluster_id: Some(cluster_id.into())
                    })
                }
                DeleteClusterConfigurationError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } => {
                    delete_cluster_configuration_failure::Error::IllegalClusterState(DeleteClusterConfigurationFailureIllegalClusterState {
                        cluster_id: Some(cluster_id.into()),
                        cluster_name: Some(cluster_name.into()),
                        actual_state: Some(actual_state.into()),
                        required_states: required_states.into_iter().map(Into::into).collect(),
                    })
                }
                DeleteClusterConfigurationError::Internal { cluster_id, cluster_name, cause } => {
                    delete_cluster_configuration_failure::Error::Internal(DeleteClusterConfigurationFailureInternal {
                        cluster_id: Some(cluster_id.into()),
                        cluster_name: Some(cluster_name.into()),
                        cause
                    })
                }
            };
            DeleteClusterConfigurationFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<DeleteClusterConfigurationFailure> for DeleteClusterConfigurationError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterConfigurationFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterConfigurationFailure, DeleteClusterConfigurationError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                delete_cluster_configuration_failure::Error::ClusterConfigurationNotFound(error) => {
                    error.try_into()?
                }
                delete_cluster_configuration_failure::Error::IllegalClusterState(error) => {
                    error.try_into()?
                }
                delete_cluster_configuration_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<DeleteClusterConfigurationFailureClusterConfigurationNotFound> for DeleteClusterConfigurationError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterConfigurationFailureClusterConfigurationNotFound) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterConfigurationFailureClusterConfigurationNotFound, DeleteClusterConfigurationError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            Ok(DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id })
        }
    }

    impl TryFrom<DeleteClusterConfigurationFailureIllegalClusterState> for DeleteClusterConfigurationError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterConfigurationFailureIllegalClusterState) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterConfigurationFailureIllegalClusterState, DeleteClusterConfigurationError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            let cluster_name: ClusterName = failure.cluster_name
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_name' not set"))?
                .try_into()?;
            let actual_state: ClusterState = failure.actual_state
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_state' not set"))?
                .try_into()?;
            let required_states = failure.required_states.into_iter()
                .map(proto::cluster::ClusterState::try_into)
                .collect::<Result<_, _>>()?;
            Ok(DeleteClusterConfigurationError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states })
        }
    }

    impl TryFrom<DeleteClusterConfigurationFailureInternal> for DeleteClusterConfigurationError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterConfigurationFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterConfigurationFailureInternal, DeleteClusterConfigurationError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            let cluster_name: ClusterName = failure.cluster_name
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_name' not set"))?
                .try_into()?;
            Ok(DeleteClusterConfigurationError::Internal { cluster_id, cluster_name, cause: failure.cause })
        }
    }

    impl From<StoreClusterDeploymentError> for StoreClusterDeploymentFailure {
        fn from(error: StoreClusterDeploymentError) -> Self {
            let proto_error = match error {
                StoreClusterDeploymentError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } => {
                    store_cluster_deployment_failure::Error::IllegalClusterState(StoreClusterDeploymentFailureIllegalClusterState {
                        cluster_id: Some(cluster_id.into()),
                        cluster_name: Some(cluster_name.into()),
                        actual_state: Some(actual_state.into()),
                        required_states: required_states.into_iter().map(Into::into).collect(),
                    })
                }
                StoreClusterDeploymentError::Internal { cluster_id, cluster_name, cause } => {
                    store_cluster_deployment_failure::Error::Internal(StoreClusterDeploymentFailureInternal {
                        cluster_id: Some(cluster_id.into()),
                        cluster_name: Some(cluster_name.into()),
                        cause
                    })
                }
            };
            StoreClusterDeploymentFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<StoreClusterDeploymentFailure> for StoreClusterDeploymentError {
        type Error = ConversionError;
        fn try_from(failure: StoreClusterDeploymentFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<StoreClusterDeploymentFailure, StoreClusterDeploymentError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                store_cluster_deployment_failure::Error::IllegalClusterState(error) => {
                    error.try_into()?
                }
                store_cluster_deployment_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<StoreClusterDeploymentFailureIllegalClusterState> for StoreClusterDeploymentError {
        type Error = ConversionError;
        fn try_from(failure: StoreClusterDeploymentFailureIllegalClusterState) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<StoreClusterDeploymentFailureIllegalClusterState, StoreClusterDeploymentError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            let cluster_name: ClusterName = failure.cluster_name
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_name' not set"))?
                .try_into()?;
            let actual_state: ClusterState = failure.actual_state
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_state' not set"))?
                .try_into()?;
            let required_states = failure.required_states.into_iter()
                .map(proto::cluster::ClusterState::try_into)
                .collect::<Result<_, _>>()?;
            Ok(StoreClusterDeploymentError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states })
        }
    }

    impl TryFrom<StoreClusterDeploymentFailureInternal> for StoreClusterDeploymentError {
        type Error = ConversionError;
        fn try_from(failure: StoreClusterDeploymentFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<StoreClusterDeploymentFailureInternal, StoreClusterDeploymentError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            let cluster_name: ClusterName = failure.cluster_name
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_name' not set"))?
                .try_into()?;
            Ok(StoreClusterDeploymentError::Internal { cluster_id, cluster_name, cause: failure.cause })
        }
    }

    impl From<DeleteClusterDeploymentError> for DeleteClusterDeploymentFailure {
        fn from(error: DeleteClusterDeploymentError) -> Self {
            let proto_error = match error {
                DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id } => {
                    delete_cluster_deployment_failure::Error::ClusterDeploymentNotFound(DeleteClusterDeploymentFailureClusterDeploymentNotFound {
                        cluster_id: Some(cluster_id.into())
                    })
                }
                DeleteClusterDeploymentError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } => {
                    delete_cluster_deployment_failure::Error::IllegalClusterState(DeleteClusterDeploymentFailureIllegalClusterState {
                        cluster_id: Some(cluster_id.into()),
                        cluster_name: Some(cluster_name.into()),
                        actual_state: Some(actual_state.into()),
                        required_states: required_states.into_iter().map(Into::into).collect(),
                    })
                }
                DeleteClusterDeploymentError::Internal { cluster_id, cluster_name, cause } => {
                    delete_cluster_deployment_failure::Error::Internal(DeleteClusterDeploymentFailureInternal {
                        cluster_id: Some(cluster_id.into()),
                        cluster_name: Some(cluster_name.into()),
                        cause
                    })
                }
            };
            DeleteClusterDeploymentFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<DeleteClusterDeploymentFailure> for DeleteClusterDeploymentError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterDeploymentFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDeploymentFailure, DeleteClusterDeploymentError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                delete_cluster_deployment_failure::Error::ClusterDeploymentNotFound(error) => {
                    error.try_into()?
                }
                delete_cluster_deployment_failure::Error::IllegalClusterState(error) => {
                    error.try_into()?
                }
                delete_cluster_deployment_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<DeleteClusterDeploymentFailureClusterDeploymentNotFound> for DeleteClusterDeploymentError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterDeploymentFailureClusterDeploymentNotFound) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDeploymentFailureClusterDeploymentNotFound, DeleteClusterDeploymentError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            Ok(DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id })
        }
    }

    impl TryFrom<DeleteClusterDeploymentFailureIllegalClusterState> for DeleteClusterDeploymentError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterDeploymentFailureIllegalClusterState) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDeploymentFailureIllegalClusterState, DeleteClusterDeploymentError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            let cluster_name: ClusterName = failure.cluster_name
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_name' not set"))?
                .try_into()?;
            let actual_state: ClusterState = failure.actual_state
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_state' not set"))?
                .try_into()?;
            let required_states = failure.required_states.into_iter()
                .map(proto::cluster::ClusterState::try_into)
                .collect::<Result<_, _>>()?;
            Ok(DeleteClusterDeploymentError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states })
        }
    }

    impl TryFrom<DeleteClusterDeploymentFailureInternal> for DeleteClusterDeploymentError {
        type Error = ConversionError;
        fn try_from(failure: DeleteClusterDeploymentFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDeploymentFailureInternal, DeleteClusterDeploymentError>;
            let cluster_id: ClusterId = failure.cluster_id
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_id' not set"))?
                .try_into()?;
            let cluster_name: ClusterName = failure.cluster_name
                .ok_or_else(|| ErrorBuilder::new("Field 'cluster_name' not set"))?
                .try_into()?;
            Ok(DeleteClusterDeploymentError::Internal { cluster_id, cluster_name, cause: failure.cause })
        }
    }

}

pub mod metadata_provider {
    tonic::include_proto!("opendut.carl.services.metadata_provider");
}

#[allow(clippy::large_enum_variant)]
pub mod peer_manager {
    use opendut_types::peer::{PeerId, PeerName};
    use opendut_types::peer::state::PeerState;
    use opendut_types::proto;
    use opendut_types::proto::{ConversionError, ConversionErrorBuilder};
    use opendut_types::topology::DeviceId;

    use crate::carl::peer::{StorePeerDescriptorError, DeletePeerDescriptorError, GetPeerDescriptorError, ListPeerDescriptorsError};

    tonic::include_proto!("opendut.carl.services.peer_manager");

    impl From<StorePeerDescriptorError> for StorePeerDescriptorFailure {
        fn from(error: StorePeerDescriptorError) -> Self {
            let proto_error = match error {
                StorePeerDescriptorError::IllegalPeerState { peer_id, peer_name, actual_state, required_states } => {
                    store_peer_descriptor_failure::Error::IllegalPeerState(StorePeerDescriptorFailureIllegalPeerState {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        actual_state: Some(actual_state.into()),
                        required_states: required_states.into_iter().map(Into::into).collect(),
                    })
                }
                StorePeerDescriptorError::IllegalDevices { peer_id, peer_name, error } => {
                    store_peer_descriptor_failure::Error::IllegalDevices(StorePeerDescriptorFailureIllegalDevices {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        error: Some(error.into()),
                    })
                }
                StorePeerDescriptorError::Internal { peer_id, peer_name, cause } => {
                    store_peer_descriptor_failure::Error::Internal(StorePeerDescriptorFailureInternal {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        cause
                    })
                }
            };
            StorePeerDescriptorFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<StorePeerDescriptorFailure> for StorePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: StorePeerDescriptorFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<StorePeerDescriptorFailure, StorePeerDescriptorError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                store_peer_descriptor_failure::Error::IllegalPeerState(error) => {
                    error.try_into()?
                }
                store_peer_descriptor_failure::Error::IllegalDevices(error) => {
                    error.try_into()?
                }
                store_peer_descriptor_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<StorePeerDescriptorFailureIllegalPeerState> for StorePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: StorePeerDescriptorFailureIllegalPeerState) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<StorePeerDescriptorFailureIllegalPeerState, StorePeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            let actual_state: PeerState = failure.actual_state
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_state' not set"))?
                .try_into()?;
            let required_states = failure.required_states.into_iter()
                .map(proto::peer::PeerState::try_into)
                .collect::<Result<_, _>>()?;
            Ok(StorePeerDescriptorError::IllegalPeerState { peer_id, peer_name, actual_state, required_states })
        }
    }

    impl TryFrom<StorePeerDescriptorFailureIllegalDevices> for StorePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: StorePeerDescriptorFailureIllegalDevices) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<StorePeerDescriptorFailureIllegalDevices, StorePeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            let error: crate::carl::peer::IllegalDevicesError = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?
                .try_into()?;
            Ok(StorePeerDescriptorError::IllegalDevices { peer_id, peer_name, error })
        }
    }

    impl TryFrom<StorePeerDescriptorFailureInternal> for StorePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: StorePeerDescriptorFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<StorePeerDescriptorFailureInternal, StorePeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            Ok(StorePeerDescriptorError::Internal { peer_id, peer_name, cause: failure.cause })
        }
    }

    impl From<DeletePeerDescriptorError> for DeletePeerDescriptorFailure {
        fn from(error: DeletePeerDescriptorError) -> Self {
            let proto_error = match error {
                DeletePeerDescriptorError::PeerNotFound { peer_id } => {
                    delete_peer_descriptor_failure::Error::PeerNotFound(DeletePeerDescriptorFailurePeerNotFound {
                        peer_id: Some(peer_id.into())
                    })
                }
                DeletePeerDescriptorError::IllegalPeerState { peer_id, peer_name, actual_state, required_states } => {
                    delete_peer_descriptor_failure::Error::IllegalPeerState(DeletePeerDescriptorFailureIllegalPeerState {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        actual_state: Some(actual_state.into()),
                        required_states: required_states.into_iter().map(Into::into).collect(),
                    })
                }
                DeletePeerDescriptorError::Internal { peer_id, peer_name, cause } => {
                    delete_peer_descriptor_failure::Error::Internal(DeletePeerDescriptorFailureInternal {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        cause
                    })
                }
            };
            DeletePeerDescriptorFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<DeletePeerDescriptorFailure> for DeletePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerDescriptorFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerDescriptorFailure, DeletePeerDescriptorError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                delete_peer_descriptor_failure::Error::PeerNotFound(error) => {
                    error.try_into()?
                }
                delete_peer_descriptor_failure::Error::IllegalPeerState(error) => {
                    error.try_into()?
                }
                delete_peer_descriptor_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<DeletePeerDescriptorFailurePeerNotFound> for DeletePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerDescriptorFailurePeerNotFound) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerDescriptorFailurePeerNotFound, DeletePeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            Ok(DeletePeerDescriptorError::PeerNotFound { peer_id })
        }
    }

    impl TryFrom<DeletePeerDescriptorFailureIllegalPeerState> for DeletePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerDescriptorFailureIllegalPeerState) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerDescriptorFailureIllegalPeerState, DeletePeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            let actual_state: PeerState = failure.actual_state
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_state' not set"))?
                .try_into()?;
            let required_states = failure.required_states.into_iter()
                .map(proto::peer::PeerState::try_into)
                .collect::<Result<_, _>>()?;
            Ok(DeletePeerDescriptorError::IllegalPeerState { peer_id, peer_name, actual_state, required_states })
        }
    }

    impl TryFrom<DeletePeerDescriptorFailureInternal> for DeletePeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerDescriptorFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerDescriptorFailureInternal, DeletePeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            Ok(DeletePeerDescriptorError::Internal { peer_id, peer_name, cause: failure.cause })
        }
    }

    impl From<crate::carl::peer::IllegalDevicesError> for IllegalDevicesError {
        fn from(error: crate::carl::peer::IllegalDevicesError) -> Self {
            match error {
                crate::carl::peer::IllegalDevicesError::DeviceAlreadyExists { device_id } => {
                    IllegalDevicesError {
                        error: Some(illegal_devices_error::Error::DeviceAlreadyExists(IllegalDevicesErrorDeviceAlreadyExists {
                            device_id: Some(device_id.into())
                        })),
                    }
                }
            }
        }
    }

    impl TryFrom<IllegalDevicesError> for crate::carl::peer::IllegalDevicesError {
        type Error = ConversionError;
        fn try_from(error: IllegalDevicesError) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<IllegalDevicesError, crate::carl::peer::IllegalDevicesError>;
            let inner = error.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            match inner {
                illegal_devices_error::Error::DeviceAlreadyExists(error) => {
                    error.try_into()
                }
            }
        }
    }

    impl TryFrom<IllegalDevicesErrorDeviceAlreadyExists> for crate::carl::peer::IllegalDevicesError {
        type Error = ConversionError;
        fn try_from(error: IllegalDevicesErrorDeviceAlreadyExists) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<IllegalDevicesErrorDeviceAlreadyExists, crate::carl::peer::IllegalDevicesError>;
            let device_id: DeviceId = error.device_id
                .ok_or_else(|| ErrorBuilder::new("Field 'device_id' not set"))?
                .try_into()?;
            Ok(crate::carl::peer::IllegalDevicesError::DeviceAlreadyExists { device_id })
        }
    }

    impl From<GetPeerDescriptorError> for GetPeerDescriptorFailure {
        fn from(error: GetPeerDescriptorError) -> Self {
            let proto_error = match error {
                GetPeerDescriptorError::PeerNotFound { peer_id } => {
                    get_peer_descriptor_failure::Error::PeerNotFound(GetPeerDescriptorFailurePeerNotFound {
                        peer_id: Some(peer_id.into()),
                    })
                }
                GetPeerDescriptorError::Internal { peer_id, cause } => {
                    get_peer_descriptor_failure::Error::Internal(GetPeerDescriptorFailureInternal {
                        peer_id: Some(peer_id.into()),
                        cause
                    })
                }
            };
            GetPeerDescriptorFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<GetPeerDescriptorFailure> for GetPeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: GetPeerDescriptorFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<GetPeerDescriptorFailure, GetPeerDescriptorError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                get_peer_descriptor_failure::Error::PeerNotFound(error) => {
                    error.try_into()?
                }
                get_peer_descriptor_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<GetPeerDescriptorFailurePeerNotFound> for GetPeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: GetPeerDescriptorFailurePeerNotFound) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<GetPeerDescriptorFailurePeerNotFound, GetPeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            Ok(GetPeerDescriptorError::PeerNotFound { peer_id })
        }
    }

    impl TryFrom<GetPeerDescriptorFailureInternal> for GetPeerDescriptorError {
        type Error = ConversionError;
        fn try_from(failure: GetPeerDescriptorFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<GetPeerDescriptorFailureInternal, GetPeerDescriptorError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            Ok(GetPeerDescriptorError::Internal{ peer_id, cause: failure.cause})
        }
    }

    impl From<ListPeerDescriptorsError> for ListPeerDescriptorsFailure {
        fn from(error: ListPeerDescriptorsError) -> Self {
            let proto_error = match error {
                ListPeerDescriptorsError::Internal { cause } => {
                    list_peer_descriptors_failure::Error::Internal(ListPeerDescriptorsFailureInternal {
                        cause
                    })
                }
            };
            ListPeerDescriptorsFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<ListPeerDescriptorsFailure> for ListPeerDescriptorsError {
        type Error = ConversionError;
        fn try_from(failure: ListPeerDescriptorsFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<ListPeerDescriptorsFailure, ListPeerDescriptorsError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                list_peer_descriptors_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<ListPeerDescriptorsFailureInternal> for ListPeerDescriptorsError {
        type Error = ConversionError;
        fn try_from(failure: ListPeerDescriptorsFailureInternal) -> Result<Self, Self::Error> {
            // type ErrorBuilder = ConversionErrorBuilder<ListPeersFailureInternal, ListPeersError>;
            Ok(ListPeerDescriptorsError::Internal{ cause: failure.cause})
        }
    }
}

pub mod peer_messaging_broker {
    tonic::include_proto!("opendut.carl.services.peer_messaging_broker");
}
