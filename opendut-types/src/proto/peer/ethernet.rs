use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.ethernet.rs"));


mod ethernet_bridge {
    use super::*;
    type Model = crate::peer::ethernet::EthernetBridge;
    type Proto = EthernetBridge;

    impl From<Model> for Proto {
        fn from(value: Model) -> Self {
            Self {
                name: Some(value.name.into())
            }
        }
    }

    impl TryFrom<Proto> for Model {
        type Error = ConversionError;

        fn try_from(value: Proto) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Proto, Model>;

            let name = value.name
                .ok_or(ErrorBuilder::field_not_set("name"))?
                .try_into()?;

            Ok(crate::peer::ethernet::EthernetBridge {
                name,
            })
        }
    }
}
