use crate::carl;
use crate::carl::cluster::{CreateClusterDescriptorError, DeleteClusterDeploymentError, DeleteClusterDescriptorError, StoreClusterDeploymentError};
use opendut_model::cluster::state::ClusterState;
use opendut_model::cluster::{ClusterId, ClusterName};
use opendut_model::conversion;
use opendut_model::proto;
use opendut_model::proto::ConversionResult;
use opendut_model::proto::{ConversionError, ConversionErrorBuilder};
use std::collections::HashMap;

tonic::include_proto!("opendut.carl.services.cluster_manager");

conversion! {
    type Model = carl::cluster::ListClusterPeerStatesResponse;
    type Proto = ListClusterPeerStatesResponse;

    fn from(value: Model) -> Proto {
        let result = match value {
            Model::Success { peer_states  } => {
                list_cluster_peer_states_response::Result::Success(ListClusterPeerStatesSuccess {
                    peer_states: peer_states.into_iter().map(|(peer_id, peer_state)| (peer_id.uuid.to_string(), peer_state.into())).collect(),
                })
            }
            Model::Failure { message  } => {
                list_cluster_peer_states_response::Result::Failure(ListClusterPeerStatesFailure {
                    cause: message
                })
            }
        };
        Proto {
            result: Some(result),
        }
    }
    fn try_from(value: Proto) -> ConversionResult<Model> {
        let result = extract!(value.result)?;
        let result = match result {
            list_cluster_peer_states_response::Result::Success(ListClusterPeerStatesSuccess { peer_states}) => {
                let peer_states = peer_states.into_iter().map(|(peer_id, peer_state)| {
                    let peer_id = opendut_model::peer::PeerId::try_from(peer_id.as_str());
                    let peer_state = opendut_model::peer::state::PeerState::try_from(peer_state);
                    match (peer_id, peer_state) {
                        (Ok(peer_id), Ok(peer_state)) => {
                            Ok((peer_id, peer_state))
                        }
                        (_, _) => {
                            Err(ErrorBuilder::message("Invalid peer state"))
                        }
                    }
                }).collect::<Result<HashMap<_, _>, ConversionError>>()?;
                Model::Success { peer_states }
            }
            list_cluster_peer_states_response::Result::Failure(ListClusterPeerStatesFailure { cause}) => {
                Model::Failure { message: cause }
            }
        };
        Ok(result)
    }
}

impl From<CreateClusterDescriptorError> for CreateClusterDescriptorFailure {
    fn from(error: CreateClusterDescriptorError) -> Self {
        let proto_error = match error {
            CreateClusterDescriptorError::Internal { cluster_id, cluster_name, cause } => {
                create_cluster_descriptor_failure::Error::Internal(CreateClusterDescriptorFailureInternal {
                    cluster_id: Some(cluster_id.into()),
                    cluster_name: Some(cluster_name.into()),
                    cause
                })
            }
        };
        CreateClusterDescriptorFailure {
            error: Some(proto_error)
        }
    }
}

impl TryFrom<CreateClusterDescriptorFailure> for CreateClusterDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: CreateClusterDescriptorFailure) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<CreateClusterDescriptorFailure, CreateClusterDescriptorError>;
        let error = failure.error
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
        let error = match error {
            create_cluster_descriptor_failure::Error::Internal(error) => {
                error.try_into()?
            }
        };
        Ok(error)
    }
}

impl TryFrom<CreateClusterDescriptorFailureInternal> for CreateClusterDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: CreateClusterDescriptorFailureInternal) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<CreateClusterDescriptorFailureInternal, CreateClusterDescriptorError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: ClusterName = failure.cluster_name
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_name"))?
            .try_into()?;
        Ok(CreateClusterDescriptorError::Internal { cluster_id, cluster_name, cause: failure.cause })
    }
}

impl From<DeleteClusterDescriptorError> for DeleteClusterDescriptorFailure {
    fn from(error: DeleteClusterDescriptorError) -> Self {
        let proto_error = match error {
            DeleteClusterDescriptorError::ClusterDescriptorNotFound { cluster_id } => {
                delete_cluster_descriptor_failure::Error::ClusterDescriptorNotFound(DeleteClusterDescriptorFailureClusterDescriptorNotFound {
                    cluster_id: Some(cluster_id.into())
                })
            }
            DeleteClusterDescriptorError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } => {
                delete_cluster_descriptor_failure::Error::IllegalClusterState(DeleteClusterDescriptorFailureIllegalClusterState {
                    cluster_id: Some(cluster_id.into()),
                    cluster_name: Some(cluster_name.into()),
                    actual_state: Some(actual_state.into()),
                    required_states: required_states.into_iter().map(Into::into).collect(),
                })
            }
            DeleteClusterDescriptorError::Internal { cluster_id, cluster_name, cause } => {
                delete_cluster_descriptor_failure::Error::Internal(DeleteClusterDescriptorFailureInternal {
                    cluster_id: Some(cluster_id.into()),
                    cluster_name: cluster_name.map(Into::into),
                    cause
                })
            }
            DeleteClusterDescriptorError::ClusterDeploymentFound { cluster_id } => {
                delete_cluster_descriptor_failure::Error::ClusterDeploymentExists(DeleteClusterDescriptorFailureClusterDeploymentExists {
                    cluster_id: Some(cluster_id.into()),
                })
            }
        };
        DeleteClusterDescriptorFailure {
            error: Some(proto_error)
        }
    }
}

impl TryFrom<DeleteClusterDescriptorFailure> for DeleteClusterDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: DeleteClusterDescriptorFailure) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDescriptorFailure, DeleteClusterDescriptorError>;
        let error = failure.error
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
        let error = match error {
            delete_cluster_descriptor_failure::Error::ClusterDescriptorNotFound(error) => {
                error.try_into()?
            }
            delete_cluster_descriptor_failure::Error::IllegalClusterState(error) => {
                error.try_into()?
            }
            delete_cluster_descriptor_failure::Error::Internal(error) => {
                error.try_into()?
            }
            delete_cluster_descriptor_failure::Error::ClusterDeploymentExists(error) => {
                error.try_into()?
            }
        };
        Ok(error)
    }
}

impl TryFrom<DeleteClusterDescriptorFailureClusterDescriptorNotFound> for DeleteClusterDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: DeleteClusterDescriptorFailureClusterDescriptorNotFound) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDescriptorFailureClusterDescriptorNotFound, DeleteClusterDescriptorError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        Ok(DeleteClusterDescriptorError::ClusterDescriptorNotFound { cluster_id })
    }
}

impl TryFrom<DeleteClusterDescriptorFailureClusterDeploymentExists> for DeleteClusterDescriptorError {
    type Error = ConversionError;

    fn try_from(failure: DeleteClusterDescriptorFailureClusterDeploymentExists) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDescriptorFailureClusterDeploymentExists, DeleteClusterDescriptorError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        Ok(DeleteClusterDescriptorError::ClusterDeploymentFound { cluster_id })
    }
}

impl TryFrom<DeleteClusterDescriptorFailureIllegalClusterState> for DeleteClusterDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: DeleteClusterDescriptorFailureIllegalClusterState) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDescriptorFailureIllegalClusterState, DeleteClusterDescriptorError>;
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
        Ok(DeleteClusterDescriptorError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states })
    }
}

impl TryFrom<DeleteClusterDescriptorFailureInternal> for DeleteClusterDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: DeleteClusterDescriptorFailureInternal) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeleteClusterDescriptorFailureInternal, DeleteClusterDescriptorError>;
        let cluster_id: ClusterId = failure.cluster_id
            .ok_or_else(|| ErrorBuilder::field_not_set("cluster_id"))?
            .try_into()?;
        let cluster_name: Option<ClusterName> = failure.cluster_name
            .map(TryInto::try_into)
            .transpose()?;
        Ok(DeleteClusterDescriptorError::Internal { cluster_id, cluster_name, cause: failure.cause })
    }
}

impl From<StoreClusterDeploymentError> for StoreClusterDeploymentFailure {
    fn from(error: StoreClusterDeploymentError) -> Self {
        let proto_error = match error {
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
