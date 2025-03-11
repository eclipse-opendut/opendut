use crate::proto::{conversion, ConversionError, ConversionErrorBuilder, ConversionResult};

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.configuration.parameter.rs"));


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

        Ok(crate::peer::configuration::parameter::DeviceInterface {
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

        Ok(crate::peer::configuration::parameter::EthernetBridge {
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

        Ok(crate::peer::configuration::parameter::Executor {
            descriptor,
        })
    }
}
