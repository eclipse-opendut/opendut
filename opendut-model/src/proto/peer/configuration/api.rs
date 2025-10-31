use std::time::SystemTime;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

opendut_util::include_proto!("opendut.model.peer.configuration.api");


conversion! {
    type Model = crate::peer::configuration::OldPeerConfiguration;
    type Proto = OldPeerConfiguration;

    fn from(value: Model) -> Proto {
        Proto {
            cluster_assignment: value.cluster_assignment.map(|assignment| assignment.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let cluster_assignment = value.cluster_assignment
            .map(TryInto::try_into)
            .transpose()?;

        Ok(Model {
            cluster_assignment,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::PeerConfiguration;
    type Proto = PeerConfiguration;

    fn from(value: Model) -> Proto {
        Proto {
            executors: value.executors.into_iter().map(From::from).collect(),
            ethernet_bridges: value.ethernet_bridges.into_iter().map(From::from).collect(),
            device_interfaces: value.device_interfaces.into_iter().map(From::from).collect(),
            gre_interfaces: value.gre_interfaces.into_iter().map(From::from).collect(),
            joined_interfaces: value.joined_interfaces.into_iter().map(From::from).collect(),
            remote_peer_connection_checks: value.remote_peer_connection_checks.into_iter().map(From::from).collect(),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Ok(Model {
            executors: value.executors.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            ethernet_bridges: value.ethernet_bridges.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            device_interfaces: value.device_interfaces.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            gre_interfaces: value.gre_interfaces.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            joined_interfaces: value.joined_interfaces.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            remote_peer_connection_checks: value.remote_peer_connection_checks.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::DeviceInterface>;
    type Proto = PeerConfigurationParameterDeviceInterface;

    fn from(model: Model) -> Proto {

        let value: crate::proto::peer::configuration::parameter::DeviceInterface = model.value.clone().into();
        let parameter = PeerConfigurationParameter::from(model);

        Proto {
            parameter: Some(parameter),
            value: Some(value),
        }
    }

    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let parameter = extract!(proto.parameter)?;

        let value: crate::peer::configuration::parameter::DeviceInterface = extract!(proto.value)?.try_into()?;

        Ok(Model {
            id: extract!(parameter.id)?.try_into()?,
            dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            target: extract!(parameter.target_state)?.into(),
            value,
        })
    }
}
conversion! {
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::Executor>;
    type Proto = PeerConfigurationParameterExecutor;

    fn from(model: Model) -> Proto {

        let value: crate::proto::peer::configuration::parameter::Executor = model.value.clone().into();
        let parameter = PeerConfigurationParameter::from(model);

        Proto {
            parameter: Some(parameter),
            value: Some(value),
        }
    }

    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let parameter = extract!(proto.parameter)?;

        let value: crate::peer::configuration::parameter::Executor = extract!(proto.value)?.try_into()?;

        Ok(Model {
            id: extract!(parameter.id)?.try_into()?,
            dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            target: extract!(parameter.target_state)?.into(),
            value,
        })
    }
}
conversion! {
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::EthernetBridge>;
    type Proto = PeerConfigurationParameterEthernetBridge;

    fn from(model: Model) -> Proto {
        let descriptor: crate::proto::peer::configuration::parameter::EthernetBridge = model.value.clone().into();
        let parameter = PeerConfigurationParameter::from(model);

        Proto {
            parameter: Some(parameter),
            value: Some(descriptor),
        }
    }

    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let parameter = extract!(proto.parameter)?;

        let value: crate::peer::configuration::parameter::EthernetBridge = extract!(proto.value)?.try_into()?;

        Ok(Model {
            id: extract!(parameter.id)?.try_into()?,
            dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            target: extract!(parameter.target_state)?.into(),
            value,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::GreInterfaceConfig>;
    type Proto = PeerConfigurationParameterGreInterfaceConfig;
    
    fn from(model: Model) -> Proto {
        let descriptor: crate::proto::peer::configuration::parameter::GreInterfaceConfig = model.value.clone().into();
        let parameter = PeerConfigurationParameter::from(model);
        
        Proto {
            parameter: Some(parameter),
            value: Some(descriptor),
        }
    }
    
    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let parameter = extract!(proto.parameter)?;
        let value: crate::peer::configuration::parameter::GreInterfaceConfig = extract!(proto.value)?.try_into()?;

        Ok(Model {
            id: extract!(parameter.id)?.try_into()?,
            dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            target: extract!(parameter.target_state)?.into(),
            value,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::InterfaceJoinConfig>;
    type Proto = PeerConfigurationParameterInterfaceJoinConfig;

    fn from(model: Model) -> Proto {
        let descriptor: crate::proto::peer::configuration::parameter::InterfaceJoinConfig = model.value.clone().into();
        let parameter = PeerConfigurationParameter::from(model);
        Proto {
            parameter: Some(parameter),
            value: Some(descriptor),
        }
    }
    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let parameter = extract!(proto.parameter)?;
        let value: crate::peer::configuration::parameter::InterfaceJoinConfig = extract!(proto.value)?.try_into()?;
        
        Ok(Model {
            id: extract!(parameter.id)?.try_into()?,
            dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            target: extract!(parameter.target_state)?.into(),
            value,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::RemotePeerConnectionCheck>;
    type Proto = PeerConfigurationParameterRemotePeerConnectionCheck;

    fn from(model: Model) -> Proto {

        let value: crate::proto::peer::configuration::parameter::RemotePeerConnectionCheck = model.value.clone().into();
        let parameter = PeerConfigurationParameter::from(model);

        Proto {
            parameter: Some(parameter),
            value: Some(value),
        }
    }

    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let parameter = extract!(proto.parameter)?;

        let value: crate::peer::configuration::parameter::RemotePeerConnectionCheck = extract!(proto.value)?.try_into()?;

        Ok(Model {
            id: extract!(parameter.id)?.try_into()?,
            dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            target: extract!(parameter.target_state)?.into(),
            value,
        })
    }
}


impl<V: crate::peer::configuration::ParameterValue> From<crate::peer::configuration::Parameter<V>> for PeerConfigurationParameter {
    fn from(value: crate::peer::configuration::Parameter<V>) -> Self {
        Self {
            id: Some(value.id.into()),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
            target_state: Some(value.target.into()),
        }
    }
}

conversion! {
    type Model = crate::peer::configuration::ParameterId;
    type Proto = PeerConfigurationParameterId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.0.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| crate::peer::configuration::ParameterId(uuid.into()))
    }
}

impl From<crate::peer::configuration::ParameterTarget> for peer_configuration_parameter::TargetState {
    fn from(value: crate::peer::configuration::ParameterTarget) -> Self {
        match value {
            crate::peer::configuration::ParameterTarget::Present => peer_configuration_parameter::TargetState::Present(PeerConfigurationParameterStateKindPresent {}),
            crate::peer::configuration::ParameterTarget::Absent => peer_configuration_parameter::TargetState::Absent(PeerConfigurationParameterStateKindAbsent {}),
        }
    }
}
impl From<peer_configuration_parameter::TargetState> for crate::peer::configuration::ParameterTarget {
    fn from(value: peer_configuration_parameter::TargetState) -> Self {
        match value {
            peer_configuration_parameter::TargetState::Present(_) => crate::peer::configuration::ParameterTarget::Present,
            peer_configuration_parameter::TargetState::Absent(_) => crate::peer::configuration::ParameterTarget::Absent,
        }
    }
}


conversion! {
    type Model = crate::peer::configuration::EdgePeerConfigurationState;
    type Proto = EdgePeerConfigurationState;

    fn from(value: Model) -> Proto {
        Proto {
            parameters: value.parameter_states.into_iter()
                .map(Into::into)
                .collect(),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Ok(Model {
            parameter_states: value.parameters.into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::EdgePeerConfigurationParameterState;
    type Proto = EdgePeerConfigurationParameterState;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            timestamp: Some(value.timestamp.into()),
            detected_state: Some(value.detected_state.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let timestamp = SystemTime::try_from(extract!(value.timestamp)?)
            .map_err(|error| ErrorBuilder::message(error.to_string()))?;
        Ok(Model {
            id: extract!(value.id)?.try_into()?,
            timestamp,
            detected_state: extract!(value.detected_state)?.try_into()?,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::ParameterEdgeDetectedStateKind;
    type Proto = edge_peer_configuration_parameter_state::DetectedState;

    fn from(value: Model) -> Proto {
        match value {
            Model::Present => { Proto::Present(PeerConfigurationParameterStateKindPresent {})}
            Model::Absent => { Proto::Absent(PeerConfigurationParameterStateKindAbsent {})}
            Model::Error(error) => {
                Proto::Error(error.into())
            }
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let result = match value {
            Proto::Present(PeerConfigurationParameterStateKindPresent {}) => Model::Present,
            Proto::Absent(PeerConfigurationParameterStateKindAbsent {}) => Model::Absent,
            Proto::Error(error) => {
                Model::Error(error.try_into()?)
            },
        };
        Ok(result)
    }
}

conversion! {
    type Model = crate::peer::configuration::PeerConfigurationParameterState;
    type Proto = PeerConfigurationParameterState;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            timestamp: Some(value.timestamp.into()),
            detected_state: Some(value.detected_state.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let timestamp = SystemTime::try_from(extract!(value.timestamp)?)
            .map_err(|error| ErrorBuilder::message(error.to_string()))?;
        Ok(Model {
            id: extract!(value.id)?.try_into()?,
            timestamp,
            detected_state: extract!(value.detected_state)?.try_into()?,
        })
    }
}

conversion! {
    // state determined by CARL when comparing EdgePeerConfigurationParameterState with PeerConfigurationParameter
    type Model = crate::peer::configuration::ParameterDetectedStateKind;
    type Proto = peer_configuration_parameter_state::DetectedState;

    fn from(value: Model) -> Proto {
        match value {
            Model::Absent => Proto::Absent(PeerConfigurationParameterStateKindAbsent {}),
            Model::Present => Proto::Present(PeerConfigurationParameterStateKindPresent {}),
            Model::Creating => Proto::Creating(PeerConfigurationParameterStateKindCreating {}),
            Model::Removing => Proto::Removing(PeerConfigurationParameterStateKindRemoving {}),
            Model::Error(error) => {
                Proto::Error(error.into())
            }
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let result = match value {
            Proto::Present(PeerConfigurationParameterStateKindPresent {}) => Model::Present,
            Proto::Absent(PeerConfigurationParameterStateKindAbsent {}) => Model::Absent,
            Proto::Creating(PeerConfigurationParameterStateKindCreating {}) => Model::Creating,
            Proto::Removing(PeerConfigurationParameterStateKindRemoving {}) => Model::Removing,
            Proto::Error(error) => {
                Model::Error(error.try_into()?)
            },
        };
        Ok(result)
    }
}

conversion! {
    type Model = crate::peer::configuration::ParameterDetectedStateError;
    type Proto = PeerConfigurationParameterStateKindError;

    fn from(value: Model) -> Proto {
        let error_kind = match value.kind {
            crate::peer::configuration::api::ParameterDetectedStateErrorKind::CreatingFailed => {
                peer_configuration_parameter_state_kind_error::Kind::CreatingFailed(PeerConfigurationParameterStateKindErrorCreatingFailed {})
            }
            crate::peer::configuration::api::ParameterDetectedStateErrorKind::RemovingFailed => {
                peer_configuration_parameter_state_kind_error::Kind::RemovingFailed(PeerConfigurationParameterStateKindErrorRemovingFailed {})
            }
            crate::peer::configuration::api::ParameterDetectedStateErrorKind::CheckPresentFailed => {
                peer_configuration_parameter_state_kind_error::Kind::CheckPresentFailed(PeerConfigurationParameterStateKindErrorCheckPresentFailed {})
            }
            crate::peer::configuration::api::ParameterDetectedStateErrorKind::CheckAbsentFailed => {
                peer_configuration_parameter_state_kind_error::Kind::CheckAbsentFailed(PeerConfigurationParameterStateKindErrorCheckAbsentFailed {})
            }
            crate::peer::configuration::api::ParameterDetectedStateErrorKind::WaitingForDependenciesFailed => {
                peer_configuration_parameter_state_kind_error::Kind::WaitingForDependencies(PeerConfigurationParameterStateKindErrorWaitingForDependencies {})
            }
        };
        let error_cause = match value.cause {
            crate::peer::configuration::api::ParameterDetectedStateErrorCause::Unclassified(message) => {
                peer_configuration_parameter_state_kind_error::Cause::Unclassified(UnclassifiedError { message })
            }
            crate::peer::configuration::api::ParameterDetectedStateErrorCause::MissingDependencies(missing_ids) => {
                peer_configuration_parameter_state_kind_error::Cause::MissingDependencies(
                    MissingDependenciesError { missing_dependencies: missing_ids.into_iter().map(Into::into).collect() }
                )
            }
        };
        PeerConfigurationParameterStateKindError { kind: Some(error_kind), cause: Some(error_cause)}
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error_kind = match extract!(value.kind)? {
            peer_configuration_parameter_state_kind_error::Kind::CreatingFailed(_) => crate::peer::configuration::api::ParameterDetectedStateErrorKind::CreatingFailed,
            peer_configuration_parameter_state_kind_error::Kind::RemovingFailed(_) => crate::peer::configuration::api::ParameterDetectedStateErrorKind::RemovingFailed,
            peer_configuration_parameter_state_kind_error::Kind::CheckPresentFailed(_) => crate::peer::configuration::api::ParameterDetectedStateErrorKind::CheckPresentFailed,
            peer_configuration_parameter_state_kind_error::Kind::CheckAbsentFailed(_) => crate::peer::configuration::api::ParameterDetectedStateErrorKind::CheckAbsentFailed,
            peer_configuration_parameter_state_kind_error::Kind::WaitingForDependencies(_) => crate::peer::configuration::api::ParameterDetectedStateErrorKind::CheckAbsentFailed,
        };
        let error_cause = match extract!(value.cause)? {
            peer_configuration_parameter_state_kind_error::Cause::Unclassified(details) => {
                crate::peer::configuration::api::ParameterDetectedStateErrorCause::Unclassified(details.message)
            }
            peer_configuration_parameter_state_kind_error::Cause::MissingDependencies(error) => {
                let missing_ids = error.missing_dependencies.into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_, _>>()?;
                crate::peer::configuration::api::ParameterDetectedStateErrorCause::MissingDependencies(missing_ids)
            }
        };
        Ok(crate::peer::configuration::api::ParameterDetectedStateError {
            kind: error_kind,
            cause: error_cause,
        })
    }
}
