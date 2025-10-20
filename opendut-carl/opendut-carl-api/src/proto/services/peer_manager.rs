use opendut_model::peer::state::PeerState;
use opendut_model::peer::{PeerId, PeerName};
use opendut_model::proto::{ConversionError, ConversionErrorBuilder, ConversionResult};
use opendut_model::topology::DeviceId;
use opendut_model::{conversion, proto};

use crate::carl::peer::{DeletePeerDescriptorError, GetPeerDescriptorError, GetPeerStateError, ListPeerDescriptorsError, ListPeerStatesError, StorePeerDescriptorError};

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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
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
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_id"))?
            .try_into()?;
        let peer_name: PeerName = failure.peer_name
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_name"))?
            .try_into()?;
        let actual_state: PeerState = failure.actual_state
            .ok_or_else(|| ErrorBuilder::field_not_set("actual_state"))?
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
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_id"))?
            .try_into()?;
        let peer_name: PeerName = failure.peer_name
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_name"))?
            .try_into()?;
        let error: crate::carl::peer::IllegalDevicesError = failure.error
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?
            .try_into()?;
        Ok(StorePeerDescriptorError::IllegalDevices { peer_id, peer_name, error })
    }
}

impl TryFrom<StorePeerDescriptorFailureInternal> for StorePeerDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: StorePeerDescriptorFailureInternal) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<StorePeerDescriptorFailureInternal, StorePeerDescriptorError>;
        let peer_id: PeerId = failure.peer_id
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_id"))?
            .try_into()?;
        let peer_name: PeerName = failure.peer_name
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_name"))?
            .try_into()?;
        Ok(StorePeerDescriptorError::Internal { peer_id, peer_name, cause: failure.cause })
    }
}


conversion! {
    type Model = crate::carl::peer::DeletePeerDescriptorError;
    type Proto = DeletePeerDescriptorFailure;
    
    fn from(value: Model) -> Proto {
        let proto_error = match value {
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
                    peer_name: peer_name.map(Into::into),
                    cause
                })
            }
            DeletePeerDescriptorError::ClusterDeploymentExists { peer_id, cluster_id } => {
                delete_peer_descriptor_failure::Error::DeploymentExists(DeletePeerDescriptorFailureDeploymentExists {
                    peer_id: Some(peer_id.into()),
                    cluster_id: Some(cluster_id.into()),
                })
            }
        };
        DeletePeerDescriptorFailure {
            error: Some(proto_error)
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = extract!(value.error)?;

        match error {
            delete_peer_descriptor_failure::Error::PeerNotFound(error) => {
                let peer_id = extract!(error.peer_id)?.try_into()?;
                Ok(Model::PeerNotFound { peer_id })
            }
            delete_peer_descriptor_failure::Error::IllegalPeerState(error) => {
                let peer_id = extract!(error.peer_id)?.try_into()?;
                let peer_name = extract!(error.peer_name)?.try_into()?;
                let actual_state = extract!(error.actual_state)?.try_into()?;
                let required_states = error.required_states.into_iter()
                    .map(proto::peer::PeerState::try_into)
                    .collect::<Result<_, _>>()?;
                Ok(Model::IllegalPeerState {
                    peer_id,
                    peer_name,
                    actual_state,
                    required_states,
                })
            }
            delete_peer_descriptor_failure::Error::Internal(error) => {
                let peer_id = extract!(error.peer_id)?.try_into()?;
                let peer_name: Option<PeerName> = error.peer_name
                    .map(TryInto::try_into)
                    .transpose()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    peer_id,
                    peer_name,
                    cause,
                })
            }
            delete_peer_descriptor_failure::Error::DeploymentExists(error) => {
                let peer_id = extract!(error.peer_id)?.try_into()?;
                let cluster_id = extract!(error.cluster_id)?.try_into()?;

                Ok(Model::ClusterDeploymentExists {
                    peer_id,
                    cluster_id,
                })
            }
        }
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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
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
            .ok_or_else(|| ErrorBuilder::field_not_set("device_id"))?
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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
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
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_id"))?
            .try_into()?;
        Ok(GetPeerDescriptorError::PeerNotFound { peer_id })
    }
}

impl TryFrom<GetPeerDescriptorFailureInternal> for GetPeerDescriptorError {
    type Error = ConversionError;
    fn try_from(failure: GetPeerDescriptorFailureInternal) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<GetPeerDescriptorFailureInternal, GetPeerDescriptorError>;
        let peer_id: PeerId = failure.peer_id
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_id"))?
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
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
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

impl From<GetPeerStateError> for GetPeerStateFailure {
    fn from(error: GetPeerStateError) -> Self {
        let proto_error = match error {
            GetPeerStateError::PeerNotFound { peer_id } => {
                get_peer_state_failure::Error::PeerNotFound(GetPeerStateFailurePeerNotFound {
                    peer_id: Some(peer_id.into()),
                })
            }
            GetPeerStateError::Internal { peer_id, cause } => {
                get_peer_state_failure::Error::Internal(GetPeerStateFailureInternal {
                    peer_id: Some(peer_id.into()),
                    cause
                })
            }
        };
        GetPeerStateFailure {
            error: Some(proto_error)
        }
    }
}

impl TryFrom<GetPeerStateFailurePeerNotFound> for GetPeerStateError {
    type Error = ConversionError;
    fn try_from(failure: GetPeerStateFailurePeerNotFound) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<GetPeerStateFailurePeerNotFound, GetPeerDescriptorError>;
        let peer_id: PeerId = failure.peer_id
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_id"))?
            .try_into()?;
        Ok(GetPeerStateError::PeerNotFound { peer_id })
    }
}

impl TryFrom<GetPeerStateFailureInternal> for GetPeerStateError {
    type Error = ConversionError;
    fn try_from(failure: GetPeerStateFailureInternal) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<GetPeerStateFailureInternal, GetPeerStateError>;
        let peer_id: PeerId = failure.peer_id
            .ok_or_else(|| ErrorBuilder::field_not_set("peer_id"))?
            .try_into()?;
        Ok(GetPeerStateError::Internal{ peer_id, cause: failure.cause})
    }
}

impl TryFrom<GetPeerStateFailure> for GetPeerStateError {
    type Error = ConversionError;
    fn try_from(failure: GetPeerStateFailure) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<GetPeerStateFailure, GetPeerStateError>;
        let error = failure.error
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
        let error = match error {
            get_peer_state_failure::Error::PeerNotFound(error) => {
                error.try_into()?
            }
            get_peer_state_failure::Error::Internal(error) => {
                error.try_into()?
            }
        };
        Ok(error)
    }
}

impl From<ListPeerStatesError> for ListPeerStatesFailure {
    fn from(error: ListPeerStatesError) -> Self {
        let proto_error = match error {
            ListPeerStatesError::Internal { cause } => {
                list_peer_states_failure::Error::Internal(ListPeerStatesFailureInternal {
                    cause
                })
            }
        };
        ListPeerStatesFailure {
            error: Some(proto_error)
        }
    }
}

impl TryFrom<ListPeerStatesFailure> for ListPeerStatesError {
    type Error = ConversionError;
    fn try_from(failure: ListPeerStatesFailure) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ListPeerStatesFailure, ListPeerStatesError>;
        let error = failure.error
            .ok_or_else(|| ErrorBuilder::field_not_set("error"))?;
        let error = match error {
            list_peer_states_failure::Error::Internal(error) => {
                error.try_into()?
            }
        };
        Ok(error)
    }
}

impl TryFrom<ListPeerStatesFailureInternal> for ListPeerStatesError {
    type Error = ConversionError;
    fn try_from(failure: ListPeerStatesFailureInternal) -> Result<Self, Self::Error> {
        Ok(ListPeerStatesError::Internal{ cause: failure.cause})
    }
}
