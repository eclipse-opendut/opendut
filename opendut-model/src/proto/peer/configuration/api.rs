use std::time::SystemTime;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

opendut_util::include_proto!("opendut.model.peer.configuration.api");


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
            can_connections: value.can_connections.into_iter().map(From::from).collect(),
            can_bridges: value.can_bridges.into_iter().map(From::from).collect(),
            can_local_routes: value.can_local_routes.into_iter().map(From::from).collect(),
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
            can_connections: value.can_connections.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            can_bridges: value.can_bridges.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            can_local_routes: value.can_local_routes.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
        })
    }
}

/// Macro to generate conversion implementations for peer configuration parameters
/// re-uses conversion! macro internally
/// # Arguments
/// * `ModelParameter` - The type of the model parameter value
/// * `ProtoParameter` - The type of the proto parameter message
#[macro_export]
macro_rules! parameter_conversion {
    (
        type ModelParameter = $ModelParameter:ty;
        type ProtoParameter = $ProtoParameter:ty;

    ) => {
        conversion! {
            type Model = $crate::peer::configuration::Parameter<$ModelParameter>;
            type Proto = $ProtoParameter;

            fn from(model: Model) -> Proto {

                let value = model.value.clone();
                let parameter = PeerConfigurationParameter::from(model);

                Proto {
                    parameter: Some(parameter),
                    value: Some(value.into()),
                }
            }
            fn try_from(proto: Proto) -> ConversionResult<Model> {
                let parameter = extract!(proto.parameter)?;
                let value: $ModelParameter = extract!(proto.value)?.try_into()?;

                Ok(Model {
                    id: extract!(parameter.id)?.try_into()?,
                    dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
                    target: extract!(parameter.target_state)?.into(),
                    value,
                })
            }
        }

    }
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::DeviceInterface;
    type ProtoParameter = PeerConfigurationParameterDeviceInterface;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::EthernetBridge;
    type ProtoParameter = PeerConfigurationParameterEthernetBridge;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::Executor;
    type ProtoParameter = PeerConfigurationParameterExecutor;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::GreInterfaceConfig;
    type ProtoParameter = PeerConfigurationParameterGreInterfaceConfig;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::InterfaceJoinConfig;
    type ProtoParameter = PeerConfigurationParameterInterfaceJoinConfig;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::RemotePeerConnectionCheck;
    type ProtoParameter = PeerConfigurationParameterRemotePeerConnectionCheck;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::CanConnection;
    type ProtoParameter = PeerConfigurationParameterCanConnection;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::CanBridge;
    type ProtoParameter = PeerConfigurationParameterCanBridge;
}

parameter_conversion! {
    type ModelParameter = crate::peer::configuration::parameter::CanLocalRoute;
    type ProtoParameter = PeerConfigurationParameterCanLocalRoute;
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::peer::configuration::ParameterId;
    use crate::proto::peer::configuration::api::{PeerConfigurationParameter, PeerConfigurationParameterCanLocalRoute, PeerConfigurationParameterStateKindPresent};
    use crate::proto::peer::configuration::api::peer_configuration_parameter::TargetState;
    use crate::proto::peer::configuration::parameter::CanLocalRoute;
    use crate::proto::util::{NetworkInterfaceName};

    #[test]
    fn test_convert_can_local_route_proto_to_model() {
        let can_bridge_name = "br-vcan-opendut".to_string();
        let can_local_route = CanLocalRoute {
            can_source_device_name: Some(NetworkInterfaceName { name: can_bridge_name.clone() }),
            can_destination_device_name: Some(NetworkInterfaceName { name: "can0".to_string() }),
        };
        let parameter = PeerConfigurationParameter {
            id: Some(ParameterId(Uuid::new_v4()).into()),
            dependencies: vec![],
            target_state: Some(TargetState::Present(PeerConfigurationParameterStateKindPresent {})),
        };
        let can_local_route_proto = PeerConfigurationParameterCanLocalRoute { parameter: Some(parameter), value: Some(can_local_route) };

        let can_local_route_model: crate::peer::configuration::Parameter<crate::peer::configuration::parameter::CanLocalRoute> =
            can_local_route_proto.clone().try_into().expect("Conversion failed");
        assert_eq!(can_local_route_model.value.can_source_device_name.name(), can_bridge_name);

        let can_local_route_proto_converted_back: PeerConfigurationParameterCanLocalRoute = can_local_route_model.into();
        assert_eq!(can_local_route_proto_converted_back, can_local_route_proto);
    }
}
