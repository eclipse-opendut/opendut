use crate::proto::{conversion, ConversionError, ConversionErrorBuilder, ConversionResult};

crate::include_proto!("opendut.types.peer.configuration.api");


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
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Ok(Model {
            executors: value.executors.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            ethernet_bridges: value.ethernet_bridges.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            device_interfaces: value.device_interfaces.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            gre_interfaces: value.gre_interfaces.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
            joined_interfaces: value.joined_interfaces.into_iter().map(TryInto::try_into).collect::<Result<_, _>>()?,
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
            target: extract!(parameter.target)?.into(),
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
            target: extract!(parameter.target)?.into(),
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
            target: extract!(parameter.target)?.into(),
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
            target: extract!(parameter.target)?.into(),
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
            target: extract!(parameter.target)?.into(),
            value,
        })
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
