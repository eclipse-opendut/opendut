use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.configuration.api.rs"));

impl From<crate::peer::configuration::OldPeerConfiguration> for OldPeerConfiguration {
    fn from(value: crate::peer::configuration::OldPeerConfiguration) -> Self {
        Self {
            cluster_assignment: value.cluster_assignment.map(|assignment| assignment.into()),
        }
    }
}
impl TryFrom<OldPeerConfiguration> for crate::peer::configuration::OldPeerConfiguration {
    type Error = ConversionError;

    fn try_from(value: OldPeerConfiguration) -> Result<Self, Self::Error> {
        let cluster_assignment = value.cluster_assignment
            .map(TryInto::try_into)
            .transpose()?;

        Ok(crate::peer::configuration::OldPeerConfiguration {
            cluster_assignment,
        })
    }
}


impl From<crate::peer::configuration::PeerConfiguration> for PeerConfiguration {
    fn from(value: crate::peer::configuration::PeerConfiguration) -> Self {
        Self {
            executors: value.executors.into_iter().map(From::from).collect(),
            ethernet_bridges: value.ethernet_bridges.into_iter().map(From::from).collect(),
            device_interfaces: value.device_interfaces.into_iter().map(From::from).collect(),
        }
    }
}
impl TryFrom<PeerConfiguration> for crate::peer::configuration::PeerConfiguration {
    type Error = ConversionError;

    fn try_from(value: PeerConfiguration) -> Result<Self, Self::Error> {
        Ok(crate::peer::configuration::PeerConfiguration {
            executors: value.executors.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            ethernet_bridges: value.ethernet_bridges.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            device_interfaces: value.device_interfaces.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
        })
    }
}

mod device_interface {
    use super::*;
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::DeviceInterface>;
    type Proto = PeerConfigurationParameterDeviceInterface;

    impl From<Model> for Proto {
        fn from(model: Model) -> Self {

            let value: crate::proto::peer::configuration::parameter::DeviceInterface = model.value.clone().into();
            let parameter = PeerConfigurationParameter::from(model);

            Self {
                parameter: Some(parameter),
                value: Some(value),
            }
        }
    }
    impl TryFrom<Proto> for Model {
        type Error = ConversionError;

        fn try_from(proto: Proto) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Proto, Model>;

            let parameter = proto.parameter
                .ok_or(ErrorBuilder::field_not_set("parameter"))?;

            let value: crate::peer::configuration::parameter::DeviceInterface = proto.value
                .ok_or(ErrorBuilder::field_not_set("value"))?
                .try_into()?;

            Ok(Self {
                id: parameter.id.ok_or(ErrorBuilder::field_not_set("id"))?.try_into()?,
                dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
                target: parameter.target.ok_or(ErrorBuilder::field_not_set("target"))?.into(),
                value,
            })
        }
    }
}
mod executor {
    use super::*;
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::Executor>;
    type Proto = PeerConfigurationParameterExecutor;

    impl From<Model> for Proto {
        fn from(model: Model) -> Self {

            let value: crate::proto::peer::configuration::parameter::Executor = model.value.clone().into();
            let parameter = PeerConfigurationParameter::from(model);

            Self {
                parameter: Some(parameter),
                value: Some(value),
            }
        }
    }
    impl TryFrom<Proto> for Model {
        type Error = ConversionError;

        fn try_from(proto: Proto) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Proto, Model>;

            let parameter = proto.parameter
                .ok_or(ErrorBuilder::field_not_set("parameter"))?;

            let value: crate::peer::configuration::parameter::Executor = proto.value
                .ok_or(ErrorBuilder::field_not_set("value"))?
                .try_into()?;

            Ok(Self {
                id: parameter.id.ok_or(ErrorBuilder::field_not_set("id"))?.try_into()?,
                dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
                target: parameter.target.ok_or(ErrorBuilder::field_not_set("target"))?.into(),
                value,
            })
        }
    }
}
mod ethernet_bridge {
    use super::*;
    type Model = crate::peer::configuration::Parameter<crate::peer::configuration::parameter::EthernetBridge>;
    type Proto = PeerConfigurationParameterEthernetBridge;

    impl From<Model> for Proto {
        fn from(model: Model) -> Self {

            let descriptor: crate::proto::peer::configuration::parameter::EthernetBridge = model.value.clone().into();
            let parameter = PeerConfigurationParameter::from(model);

            Self {
                parameter: Some(parameter),
                value: Some(descriptor),
            }
        }
    }
    impl TryFrom<Proto> for Model {
        type Error = ConversionError;

        fn try_from(proto: Proto) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Proto, Model>;

            let parameter = proto.parameter
                .ok_or(ErrorBuilder::field_not_set("parameter"))?;

            let value: crate::peer::configuration::parameter::EthernetBridge = proto.value
                .ok_or(ErrorBuilder::field_not_set("value"))?
                .try_into()?;

            Ok(Self {
                id: parameter.id.ok_or(ErrorBuilder::field_not_set("id"))?.try_into()?,
                dependencies: parameter.dependencies.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
                target: parameter.target.ok_or(ErrorBuilder::field_not_set("target"))?.into(),
                value,
            })
        }
    }
}

impl<V: crate::peer::configuration::ParameterValue> From<crate::peer::configuration::Parameter<V>> for PeerConfigurationParameter {
    fn from(value: crate::peer::configuration::Parameter<V>) -> Self {
        Self {
            id: Some(value.id.into()),
            dependencies: value.dependencies.into_iter().map(Into::into).collect(),
            target: Some(value.target.into()),
        }
    }
}

impl From<crate::peer::configuration::ParameterId> for PeerConfigurationParameterId {
    fn from(value: crate::peer::configuration::ParameterId) -> Self {
        Self {
            uuid: Some(value.0.into())
        }
    }
}
impl TryFrom<PeerConfigurationParameterId> for crate::peer::configuration::ParameterId {
    type Error = ConversionError;

    fn try_from(value: PeerConfigurationParameterId) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerConfigurationParameterId, crate::peer::configuration::ParameterId>;

        value.uuid
            .ok_or(ErrorBuilder::field_not_set("uuid"))
            .map(|uuid| Self(uuid.into()))
    }
}

impl From<crate::peer::configuration::ParameterTarget> for peer_configuration_parameter::Target {
    fn from(value: crate::peer::configuration::ParameterTarget) -> Self {
        match value {
            crate::peer::configuration::ParameterTarget::Present => peer_configuration_parameter::Target::Present(PeerConfigurationParameterTargetPresent {}),
            crate::peer::configuration::ParameterTarget::Absent => peer_configuration_parameter::Target::Absent(PeerConfigurationParameterTargetAbsent {}),
        }
    }
}
impl From<peer_configuration_parameter::Target> for crate::peer::configuration::ParameterTarget {
    fn from(value: peer_configuration_parameter::Target) -> Self {
        match value {
            peer_configuration_parameter::Target::Present(_) => crate::peer::configuration::ParameterTarget::Present,
            peer_configuration_parameter::Target::Absent(_) => crate::peer::configuration::ParameterTarget::Absent,
        }
    }
}
