use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

opendut_util::include_proto!("opendut.model.peer.configuration.parameter");


conversion! {
    type Model = crate::peer::configuration::parameter::DeviceInterface;
    type Proto = DeviceInterface;

    fn from(value: Model) -> Proto {
        Proto {
            descriptor: Some(value.descriptor.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let descriptor = extract!(value.descriptor)?.try_into()?;

        Ok(Model {
            descriptor,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::EthernetBridge;
    type Proto = EthernetBridge;

    fn from(value: Model) -> Proto {
        Proto {
            name: Some(value.name.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let name = extract!(value.name)?.try_into()?;

        Ok(Model {
            name,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::Executor;
    type Proto = Executor;

    fn from(value: Model) -> Proto {
        Proto {
            descriptor: Some(value.descriptor.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let descriptor = extract!(value.descriptor)?
            .try_into()?;

        Ok(Model {
            descriptor,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::GreInterfaceConfig;
    type Proto = GreInterfaceConfig;
    
    fn from(value: Model) -> Proto {
        Proto {
            local_ip: Some(value.local_ip.into()),
            remote_ip: Some(value.remote_ip.into()),
        }
    }
    
    fn try_from(value: Proto) -> ConversionResult<Model> {
        let local_ip = std::net::Ipv4Addr::try_from(extract!(value.local_ip)?)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))?;

        let remote_ip = std::net::Ipv4Addr::try_from(extract!(value.remote_ip)?)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))?;

        Ok(Model {
            local_ip,
            remote_ip,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::InterfaceJoinConfig;
    type Proto = InterfaceJoinConfig;
    
    fn from(value: Model) -> Proto {
        Proto {
            name: Some(value.name.into()),
            bridge: Some(value.bridge.into()),
        }
    }
    
    fn try_from(value: Proto) -> ConversionResult<Model> {
        let name = extract!(value.name)?.try_into()?;
        let bridge = extract!(value.bridge)?.try_into()?;
        Ok(Model {
            name,
            bridge,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::RemotePeerConnectionCheck;
    type Proto = RemotePeerConnectionCheck;

    fn from(value: Model) -> Proto {
        Proto {
            remote_peer_id: Some(value.remote_peer_id.into()),
            remote_ip: Some(value.remote_ip.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let remote_peer_id = extract!(value.remote_peer_id)?.try_into()?;
        let remote_ip = extract!(value.remote_ip)?.try_into()?;

        Ok(Model {
            remote_peer_id,
            remote_ip,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::CanConnection;
    type Proto = CanConnection;

    fn from(value: Model) -> Proto {
        Proto {
            remote_peer_id: Some(value.remote_peer_id.into()),
            remote_ip: Some(value.remote_ip.into()),
            remote_port: Some(value.remote_port.into()),
            local_port: Some(value.local_port.into()),
            local_is_server: value.local_is_server,
            buffer_timeout_microseconds: value.buffer_timeout_microseconds,
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let remote_peer_id = extract!(value.remote_peer_id)?.try_into()?;
        let remote_ip = extract!(value.remote_ip)?.try_into()?;
        let remote_port = extract!(value.remote_port)?.try_into()?;
        let local_port = extract!(value.local_port)?.try_into()?;

        Ok(Model {
            remote_peer_id,
            remote_ip,
            remote_port,
            local_port,
            local_is_server: value.local_is_server,
            buffer_timeout_microseconds: value.buffer_timeout_microseconds,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::CanBridge;
    type Proto = CanBridge;

    fn from(value: Model) -> Proto {
        Proto {
            name: Some(value.name.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let name = extract!(value.name)?.try_into()?;

        Ok(Model {
            name,
        })
    }
}

conversion! {
    type Model = crate::peer::configuration::parameter::CanLocalRoute;
    type Proto = CanLocalRoute;

    fn from(value: Model) -> Proto {
        Proto {
            bridge_name: Some(value.bridge_name.into()),
            can_device_name: Some(value.can_device_name.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let bridge_name = extract!(value.bridge_name)?.try_into()?;
        let can_device_name = extract!(value.can_device_name)?.try_into()?;

        Ok(Model {
            bridge_name,
            can_device_name,
        })
    }
}
