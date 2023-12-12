pub mod cluster_manager {
    use opendut_types::cluster::{ClusterId, ClusterName};
    use opendut_types::cluster::state::ClusterState;
    use opendut_types::proto::{ConversionError, ConversionErrorBuilder};

    use crate::carl::cluster::{CreateClusterConfigurationError, DeleteClusterConfigurationError};

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
                        required_states: required_states.into(),
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
            let required_states = failure.required_states
                .try_into()?;
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
}

pub mod metadata_provider {
    tonic::include_proto!("opendut.carl.services.metadata_provider");
}

pub mod peer_manager {
    use opendut_types::peer::{PeerId, PeerName};
    use opendut_types::peer::state::PeerState;
    use opendut_types::proto::{ConversionError, ConversionErrorBuilder};
    use opendut_types::topology::DeviceId;

    use crate::carl::peer::{CreatePeerError, DeletePeerError, GetPeerError, ListPeersError};

    tonic::include_proto!("opendut.carl.services.peer_manager");

    impl From<CreatePeerError> for CreatePeerFailure {
        fn from(error: CreatePeerError) -> Self {
            let proto_error = match error {
                CreatePeerError::PeerAlreadyExists { actual_id, actual_name, other_id, other_name } => {
                    create_peer_failure::Error::PeerAlreadyExists(CreatePeerFailurePeerAlreadyExists {
                        actual_id: Some(actual_id.into()),
                        actual_name: Some(actual_name.into()),
                        other_id: Some(other_id.into()),
                        other_name: Some(other_name.into()),
                    })
                }
                CreatePeerError::IllegalDevices { peer_id, peer_name, error } => {
                    create_peer_failure::Error::IllegalDevices(CreatePeerFailureIllegalDevices {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        error: Some(error.into()),
                    })
                }
                CreatePeerError::Internal { peer_id, peer_name, cause } => {
                    create_peer_failure::Error::Internal(CreatePeerFailureInternal {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        cause
                    })
                }
            };
            CreatePeerFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<CreatePeerFailure> for CreatePeerError {
        type Error = ConversionError;
        fn try_from(failure: CreatePeerFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<CreatePeerFailure, CreatePeerError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                create_peer_failure::Error::PeerAlreadyExists(error) => {
                    error.try_into()?
                }
                create_peer_failure::Error::IllegalDevices(error) => {
                    error.try_into()?
                }
                create_peer_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<CreatePeerFailurePeerAlreadyExists> for CreatePeerError {
        type Error = ConversionError;
        fn try_from(failure: CreatePeerFailurePeerAlreadyExists) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<CreatePeerFailurePeerAlreadyExists, CreatePeerError>;
            let actual_id: PeerId = failure.actual_id
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_id' not set"))?
                .try_into()?;
            let actual_name: PeerName = failure.actual_name
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_name' not set"))?
                .try_into()?;
            let other_id: PeerId = failure.other_id
                .ok_or_else(|| ErrorBuilder::new("Field 'other_id' not set"))?
                .try_into()?;
            let other_name: PeerName = failure.other_name
                .ok_or_else(|| ErrorBuilder::new("Field 'other_name' not set"))?
                .try_into()?;
            Ok(CreatePeerError::PeerAlreadyExists { actual_id, actual_name, other_id, other_name })
        }
    }

    impl TryFrom<CreatePeerFailureIllegalDevices> for CreatePeerError {
        type Error = ConversionError;
        fn try_from(failure: CreatePeerFailureIllegalDevices) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<CreatePeerFailureIllegalDevices, CreatePeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            let error: crate::carl::peer::RegisterDevicesError = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?
                .try_into()?;
            Ok(CreatePeerError::IllegalDevices { peer_id, peer_name, error })
        }
    }

    impl TryFrom<CreatePeerFailureInternal> for CreatePeerError {
        type Error = ConversionError;
        fn try_from(failure: CreatePeerFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<CreatePeerFailureInternal, CreatePeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            Ok(CreatePeerError::Internal { peer_id, peer_name, cause: failure.cause })
        }
    }

    impl From<DeletePeerError> for DeletePeerFailure {
        fn from(error: DeletePeerError) -> Self {
            let proto_error = match error {
                DeletePeerError::PeerNotFound { peer_id } => {
                    delete_peer_failure::Error::PeerNotFound(DeletePeerFailurePeerNotFound {
                        peer_id: Some(peer_id.into())
                    })
                }
                DeletePeerError::IllegalPeerState { peer_id, peer_name, actual_state, required_states } => {
                    delete_peer_failure::Error::IllegalPeerState(DeletePeerFailureIllegalPeerState {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        actual_state: Some(actual_state.into()),
                        required_states: required_states.into(),
                    })
                }
                DeletePeerError::IllegalDevices { peer_id, peer_name, error } => {
                    delete_peer_failure::Error::IllegalDevices(DeletePeerFailureIllegalDevices {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        error: Some(error.into()),
                    })
                }
                DeletePeerError::Internal { peer_id, peer_name, cause } => {
                    delete_peer_failure::Error::Internal(DeletePeerFailureInternal {
                        peer_id: Some(peer_id.into()),
                        peer_name: Some(peer_name.into()),
                        cause
                    })
                }
            };
            DeletePeerFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<DeletePeerFailure> for DeletePeerError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerFailure, DeletePeerError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                delete_peer_failure::Error::PeerNotFound(error) => {
                    error.try_into()?
                }
                delete_peer_failure::Error::IllegalPeerState(error) => {
                    error.try_into()?
                }
                delete_peer_failure::Error::IllegalDevices(error) => {
                    error.try_into()?
                }
                delete_peer_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<DeletePeerFailurePeerNotFound> for DeletePeerError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerFailurePeerNotFound) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerFailurePeerNotFound, DeletePeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            Ok(DeletePeerError::PeerNotFound { peer_id })
        }
    }

    impl TryFrom<DeletePeerFailureIllegalPeerState> for DeletePeerError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerFailureIllegalPeerState) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerFailureIllegalPeerState, DeletePeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            let actual_state: PeerState = failure.actual_state
                .ok_or_else(|| ErrorBuilder::new("Field 'actual_state' not set"))?
                .try_into()?;
            let required_states = failure.required_states
                .try_into()?;
            Ok(DeletePeerError::IllegalPeerState { peer_id, peer_name, actual_state, required_states })
        }
    }

    impl TryFrom<DeletePeerFailureIllegalDevices> for DeletePeerError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerFailureIllegalDevices) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerFailureIllegalDevices, DeletePeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            let error: crate::carl::peer::UnregisterDevicesError = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?
                .try_into()?;
            Ok(DeletePeerError::IllegalDevices { peer_id, peer_name, error })
        }
    }

    impl TryFrom<DeletePeerFailureInternal> for DeletePeerError {
        type Error = ConversionError;
        fn try_from(failure: DeletePeerFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeletePeerFailureInternal, DeletePeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            let peer_name: PeerName = failure.peer_name
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_name' not set"))?
                .try_into()?;
            Ok(DeletePeerError::Internal { peer_id, peer_name, cause: failure.cause })
        }
    }

    impl From<crate::carl::peer::RegisterDevicesError> for RegisterDevicesError {
        fn from(error: crate::carl::peer::RegisterDevicesError) -> Self {
            match error {
                crate::carl::peer::RegisterDevicesError::DeviceAlreadyExists { device_id } => {
                    RegisterDevicesError {
                        error: Some(register_devices_error::Error::DeviceAlreadyExists(RegisterDevicesErrorDeviceAlreadyExists {
                            device_id: Some(device_id.into())
                        })),
                    }
                }
            }
        }
    }

    impl TryFrom<RegisterDevicesError> for crate::carl::peer::RegisterDevicesError {
        type Error = ConversionError;
        fn try_from(error: RegisterDevicesError) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<RegisterDevicesError, crate::carl::peer::RegisterDevicesError>;
            let inner = error.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            match inner {
                register_devices_error::Error::DeviceAlreadyExists(error) => {
                    error.try_into()
                }
            }
        }
    }

    impl TryFrom<RegisterDevicesErrorDeviceAlreadyExists> for crate::carl::peer::RegisterDevicesError {
        type Error = ConversionError;
        fn try_from(error: RegisterDevicesErrorDeviceAlreadyExists) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<RegisterDevicesErrorDeviceAlreadyExists, crate::carl::peer::RegisterDevicesError>;
            let device_id: DeviceId = error.device_id
                .ok_or_else(|| ErrorBuilder::new("Field 'device_id' not set"))?
                .try_into()?;
            Ok(crate::carl::peer::RegisterDevicesError::DeviceAlreadyExists { device_id })
        }
    }

    impl From<crate::carl::peer::UnregisterDevicesError> for UnregisterDevicesError {
        fn from(error: crate::carl::peer::UnregisterDevicesError) -> Self {
            match error {
                crate::carl::peer::UnregisterDevicesError::DeviceNotFound { device_id } => {
                    UnregisterDevicesError {
                        error: Some(unregister_devices_error::Error::DeviceNotFound(UnregisterDevicesErrorDeviceNotFound {
                            device_id: Some(device_id.into())
                        })),
                    }
                }
            }
        }
    }

    impl TryFrom<UnregisterDevicesError> for crate::carl::peer::UnregisterDevicesError {
        type Error = ConversionError;
        fn try_from(error: UnregisterDevicesError) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<UnregisterDevicesError, crate::carl::peer::UnregisterDevicesError>;
            let inner = error.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            match inner {
                unregister_devices_error::Error::DeviceNotFound(error) => {
                    error.try_into()
                }
            }
        }
    }

    impl TryFrom<UnregisterDevicesErrorDeviceNotFound> for crate::carl::peer::UnregisterDevicesError {
        type Error = ConversionError;
        fn try_from(error: UnregisterDevicesErrorDeviceNotFound) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<UnregisterDevicesErrorDeviceNotFound, crate::carl::peer::UnregisterDevicesError>;
            let device_id: DeviceId = error.device_id
                .ok_or_else(|| ErrorBuilder::new("Field 'device_id' not set"))?
                .try_into()?;
            Ok(crate::carl::peer::UnregisterDevicesError::DeviceNotFound { device_id })
        }
    }

    impl From<GetPeerError> for GetPeerFailure {
        fn from(error: GetPeerError) -> Self {
            let proto_error = match error {
                GetPeerError::PeerNotFound { peer_id } => {
                    get_peer_failure::Error::PeerNotFound(GetPeerFailurePeerNotFound {
                        peer_id: Some(peer_id.into()),
                    })
                }
                GetPeerError::Internal { peer_id, cause } => {
                    get_peer_failure::Error::Internal(GetPeerFailureInternal {
                        peer_id: Some(peer_id.into()),
                        cause
                    })
                }
            };
            GetPeerFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<GetPeerFailure> for GetPeerError {
        type Error = ConversionError;
        fn try_from(failure: GetPeerFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<GetPeerFailure, GetPeerError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                get_peer_failure::Error::PeerNotFound(error) => {
                    error.try_into()?
                }
                get_peer_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<GetPeerFailurePeerNotFound> for GetPeerError {
        type Error = ConversionError;
        fn try_from(failure: GetPeerFailurePeerNotFound) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<GetPeerFailurePeerNotFound, GetPeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            Ok(GetPeerError::PeerNotFound { peer_id })
        }
    }

    impl TryFrom<GetPeerFailureInternal> for GetPeerError {
        type Error = ConversionError;
        fn try_from(failure: GetPeerFailureInternal) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<GetPeerFailureInternal, GetPeerError>;
            let peer_id: PeerId = failure.peer_id
                .ok_or_else(|| ErrorBuilder::new("Field 'peer_id' not set"))?
                .try_into()?;
            Ok(GetPeerError::Internal{ peer_id, cause: failure.cause})
        }
    }

    impl From<ListPeersError> for ListPeersFailure {
        fn from(error: ListPeersError) -> Self {
            let proto_error = match error {
                ListPeersError::Internal { cause } => {
                    list_peers_failure::Error::Internal(ListPeersFailureInternal {
                        cause
                    })
                }
            };
            ListPeersFailure {
                error: Some(proto_error)
            }
        }
    }

    impl TryFrom<ListPeersFailure> for ListPeersError {
        type Error = ConversionError;
        fn try_from(failure: ListPeersFailure) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<ListPeersFailure, ListPeersError>;
            let error = failure.error
                .ok_or_else(|| ErrorBuilder::new("Field 'error' not set"))?;
            let error = match error {
                list_peers_failure::Error::Internal(error) => {
                    error.try_into()?
                }
            };
            Ok(error)
        }
    }

    impl TryFrom<ListPeersFailureInternal> for ListPeersError {
        type Error = ConversionError;
        fn try_from(failure: ListPeersFailureInternal) -> Result<Self, Self::Error> {
            // type ErrorBuilder = ConversionErrorBuilder<ListPeersFailureInternal, ListPeersError>;
            Ok(ListPeersError::Internal{ cause: failure.cause})
        }
    }
}

pub mod peer_messaging_broker {
    tonic::include_proto!("opendut.carl.services.peer_messaging_broker");
}
