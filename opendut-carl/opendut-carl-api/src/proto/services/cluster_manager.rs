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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
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
            .ok_or_else(|| ErrorBuilder::field_not_set("actual_id"))?
            .try_into()?;
        let actual_name: ClusterName = failure.actual_name
            .ok_or_else(|| ErrorBuilder::field_not_set("actual_name"))?
            .try_into()?;
        let other_id: ClusterId = failure.other_id
            .ok_or_else(|| ErrorBuilder::field_not_set("other_id"))?
            .try_into()?;
        let other_name: ClusterName = failure.other_name
            .ok_or_else(|| ErrorBuilder::field_not_set("other_name"))?
            .try_into()?;
        Ok(CreateClusterConfigurationError::ClusterConfigurationAlreadyExists { actual_id, actual_name, other_id, other_name })
    }
}

impl TryFrom<CreateClusterConfigurationFailureInternal> for CreateClusterConfigurationError {
    type Error = ConversionError;
    fn try_from(failure: CreateClusterConfigurationFailureInternal) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<CreateClusterConfigurationFailureInternal, CreateClusterConfigurationError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: ClusterName = failure.cluster_name
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_name"))?
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
                    cluster_name: cluster_name.map(Into::into),
                    cause
                })
            }
            DeleteClusterConfigurationError::ClusterDeploymentFound { cluster_id } => {
                delete_cluster_configuration_failure::Error::ClusterDeploymentExists(DeleteClusterConfigurationFailureClusterDeploymentExists {
                    cluster_id: Some(cluster_id.into()),
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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
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
            delete_cluster_configuration_failure::Error::ClusterDeploymentExists(error) => {
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
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        Ok(DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id })
    }
}

impl TryFrom<DeleteClusterConfigurationFailureClusterDeploymentExists> for DeleteClusterConfigurationError {
    type Error = ConversionError;

    fn try_from(failure: DeleteClusterConfigurationFailureClusterDeploymentExists) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterConfigurationFailureClusterDeploymentExists, DeleteClusterConfigurationError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        Ok(DeleteClusterConfigurationError::ClusterDeploymentFound { cluster_id })
    }
}

impl TryFrom<DeleteClusterConfigurationFailureIllegalClusterState> for DeleteClusterConfigurationError {
    type Error = ConversionError;
    fn try_from(failure: DeleteClusterConfigurationFailureIllegalClusterState) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterConfigurationFailureIllegalClusterState, DeleteClusterConfigurationError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: ClusterName = failure.cluster_name
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_name"))?
            .try_into()?;
        let actual_state: ClusterState = failure.actual_state
            .ok_or_else(|| ErrorBuilder::field_not_set("actual_state"))?
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
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: Option<ClusterName> = failure.cluster_name
            .map(TryInto::try_into)
            .transpose()?;
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
                    cluster_name: cluster_name.map(|name| name.into()),
                    cause
                })
            }
            StoreClusterDeploymentError::IllegalPeerState { cluster_id, cluster_name, invalid_peers } => {
                store_cluster_deployment_failure::Error::IllegalPeerState(StoreClusterDeploymentFailureIllegalPeerState {
                    cluster_id: Some(cluster_id.into()),
                    cluster_name: cluster_name.map(|name| name.into()),
                    invalid_peers: invalid_peers.into_iter().map(Into::into).collect(),
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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
        let error = match error {
            store_cluster_deployment_failure::Error::IllegalClusterState(error) => {
                error.try_into()?
            }
            store_cluster_deployment_failure::Error::Internal(error) => {
                error.try_into()?
            }
            store_cluster_deployment_failure::Error::IllegalPeerState(error) => {
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
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: ClusterName = failure.cluster_name
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_name"))?
            .try_into()?;
        let actual_state: ClusterState = failure.actual_state
            .ok_or_else(|| ErrorBuilder::field_not_set("actual_state"))?
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
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: Option<ClusterName> = failure.cluster_name
            .map(TryInto::try_into)
            .transpose()?;
        Ok(StoreClusterDeploymentError::Internal { cluster_id, cluster_name, cause: failure.cause })
    }
}

impl TryFrom<StoreClusterDeploymentFailureIllegalPeerState> for StoreClusterDeploymentError {
    type Error = ConversionError;
    fn try_from(failure: StoreClusterDeploymentFailureIllegalPeerState) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<StoreClusterDeploymentFailureInternal, StoreClusterDeploymentError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: Option<ClusterName> = failure.cluster_name
            .map(TryInto::try_into)
            .transpose()?;
        let invalid_peers = failure.invalid_peers.into_iter()
            .map(proto::peer::PeerId::try_into)
            .collect::<Result<_, _>>()?;
        Ok(StoreClusterDeploymentError::IllegalPeerState { cluster_id, cluster_name, invalid_peers })
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
                    cluster_name: cluster_name.map(Into::into),
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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
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
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        Ok(DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id })
    }
}

impl TryFrom<DeleteClusterDeploymentFailureIllegalClusterState> for DeleteClusterDeploymentError {
    type Error = ConversionError;
    fn try_from(failure: DeleteClusterDeploymentFailureIllegalClusterState) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDeploymentFailureIllegalClusterState, DeleteClusterDeploymentError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: ClusterName = failure.cluster_name
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_name"))?
            .try_into()?;
        let actual_state: ClusterState = failure.actual_state
            .ok_or_else(|| ErrorBuilder::field_not_set("actual_state"))?
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
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: Option<ClusterName> = failure.cluster_name
            .map(TryInto::try_into)
            .transpose()?;
        Ok(DeleteClusterDeploymentError::Internal { cluster_id, cluster_name, cause: failure.cause })
    }
}
