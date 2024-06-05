use accessory_descriptor::Model;

use crate::{proto::{ConversionError, ConversionErrorBuilder}, topology::AccessoryModel};

include!(concat!(env!("OUT_DIR"), "/opendut.types.topology.rs"));

impl From<crate::topology::DeviceId> for DeviceId {
    fn from(value: crate::topology::DeviceId) -> Self {
        Self {
            uuid: Some(value.0.into()),
        }
    }
}

impl TryFrom<DeviceId> for crate::topology::DeviceId {
    type Error = ConversionError;

    fn try_from(value: DeviceId) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeviceId, crate::topology::DeviceId>;

        value
            .uuid
            .ok_or(ErrorBuilder::field_not_set("uuid"))
            .map(|uuid| Self(uuid.into()))
    }
}

impl From<uuid::Uuid> for DeviceId {
    fn from(value: uuid::Uuid) -> Self {
        Self {
            uuid: Some(value.into()),
        }
    }
}

impl From<crate::topology::Topology> for Topology {
    fn from(value: crate::topology::Topology) -> Self {
        Self {
            device_descriptors: value
                .devices
                .into_iter()
                .map(DeviceDescriptor::from)
                .collect(),
        }
    }
}

impl TryFrom<Topology> for crate::topology::Topology {
    type Error = ConversionError;

    fn try_from(value: Topology) -> Result<Self, Self::Error> {
        value
            .device_descriptors
            .into_iter()
            .map(DeviceDescriptor::try_into)
            .collect::<Result<_, _>>()
            .map(|devices| Self { devices })
    }
}

impl From<crate::topology::DeviceName> for DeviceName {
    fn from(value: crate::topology::DeviceName) -> Self {
        Self {
            value: String::from(value.value()),
        }
    }
}

impl TryFrom<DeviceName> for crate::topology::DeviceName {
    type Error = ConversionError;

    fn try_from(value: DeviceName) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeviceName, crate::topology::DeviceName>;

        crate::topology::DeviceName::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl TryFrom<DeviceDescription> for crate::topology::DeviceDescription {
    type Error = ConversionError;

    fn try_from(value: DeviceDescription) -> Result<Self, Self::Error> {
        type ErrorBuilder =
            ConversionErrorBuilder<DeviceDescription, crate::topology::DeviceDescription>;

        crate::topology::DeviceDescription::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl TryFrom<DeviceTag> for crate::topology::DeviceTag {
    type Error = ConversionError;

    fn try_from(value: DeviceTag) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeviceTag, crate::topology::DeviceTag>;

        crate::topology::DeviceTag::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<Option<crate::topology::DeviceDescription>> for DeviceDescription {
    fn from(value: Option<crate::topology::DeviceDescription>) -> Self {
        Self {
            value: String::from(value.unwrap_or_default().value()),
        }
    }
}

impl From<crate::topology::DeviceTag> for DeviceTag {
    fn from(value: crate::topology::DeviceTag) -> Self {
        Self {
            value: String::from(value.value()),
        }
    }
}

impl From<crate::topology::DeviceDescriptor> for DeviceDescriptor {
    fn from(value: crate::topology::DeviceDescriptor) -> Self {
        Self {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            description: Some(value.description.into()),
            interface: Some(value.interface.into()),
            tags: value.tags.into_iter().map(|value| value.into()).collect(),
            accessories: value.accessories.into_iter().map(|value| value.into()).collect(),
        }
    }
}

impl TryFrom<DeviceDescriptor> for crate::topology::DeviceDescriptor {
    type Error = ConversionError;

    fn try_from(value: DeviceDescriptor) -> Result<Self, Self::Error> {
        type ErrorBuilder =
            ConversionErrorBuilder<DeviceDescriptor, crate::topology::DeviceDescriptor>;

        let device_id: crate::topology::DeviceId = value
            .id
            .ok_or(ErrorBuilder::field_not_set("id"))?
            .try_into()?;
        let device_name: crate::topology::DeviceName = value
            .name
            .ok_or(ErrorBuilder::field_not_set("name"))?
            .try_into()?;
        let device_description: Option<crate::topology::DeviceDescription> =
            value.description.map(TryFrom::try_from).transpose()?;
        let interface: crate::util::net::NetworkInterfaceDescriptor = value
            .interface
            .ok_or(ErrorBuilder::field_not_set("interface"))?
            .try_into()?;
        let device_tags: Vec<crate::topology::DeviceTag> = value
            .tags
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()?;
        let accessories: Vec<crate::topology::AccessoryDescriptor> = value
            .accessories
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            id: device_id,
            name: device_name,
            description: device_description,
            interface,
            tags: device_tags,
            accessories,
        })
    }
}


impl From<crate::topology::AccessoryId> for AccessoryId {
    fn from(value: crate::topology::AccessoryId) -> Self {
        Self {
            uuid: Some(value.0.into()),
        }
    }
}

impl TryFrom<AccessoryId> for crate::topology::AccessoryId {
    type Error = ConversionError;

    fn try_from(value: AccessoryId) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<AccessoryId, crate::topology::AccessoryId>;

        value
            .uuid
            .ok_or(ErrorBuilder::field_not_set("uuid"))
            .map(|uuid| Self(uuid.into()))
    }
}

impl From<uuid::Uuid> for AccessoryId {
    fn from(value: uuid::Uuid) -> Self {
        Self {
            uuid: Some(value.into()),
        }
    }
}

impl From<crate::topology::AccessoryName> for AccessoryName {
    fn from(value: crate::topology::AccessoryName) -> Self {
        Self {
            value: String::from(value.value()),
        }
    }
}

impl TryFrom<AccessoryName> for crate::topology::AccessoryName {
    type Error = ConversionError;

    fn try_from(value: AccessoryName) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<AccessoryName, crate::topology::AccessoryName>;

        crate::topology::AccessoryName::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<Option<crate::topology::AccessoryDescription>> for AccessoryDescription {
    fn from(value: Option<crate::topology::AccessoryDescription>) -> Self {
        Self {
            value: String::from(value.unwrap_or_default().value()),
        }
    }
}

impl TryFrom<AccessoryDescription> for crate::topology::AccessoryDescription {
    type Error = ConversionError;

    fn try_from(value: AccessoryDescription) -> Result<Self, Self::Error> {
        type ErrorBuilder =
            ConversionErrorBuilder<AccessoryDescription, crate::topology::AccessoryDescription>;

        crate::topology::AccessoryDescription::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

// impl From<Option<crate::topology::AccessoryModel>> for AccessoryModel {
//     fn from(value: Option<crate::topology::AccessoryModel>) -> Self {
//         Self {
//             value: String::from(value.unwrap_or_default().value()),
//         }
//     }
// }

// impl TryFrom<AccessoryModel> for crate::topology::AccessoryModel {
//     type Error = ConversionError;

//     fn try_from(value: AccessoryModel) -> Result<Self, Self::Error> {
//         type ErrorBuilder =
//             ConversionErrorBuilder<AccessoryModel, crate::topology::AccessoryModel>;

//         crate::topology::AccessoryModel::try_from(value.value)
//             .map_err(|cause| ErrorBuilder::message(cause.to_string()))
//     }
// }



impl From<crate::topology::AccessoryDescriptor> for AccessoryDescriptor {
    fn from(value: crate::topology::AccessoryDescriptor) -> Self {
        let model = match value.model {
            AccessoryModel::MansonHcs3304 { 
                serial_port 
            } => Model::MansonHcs3304(MansonHcs3304 { serial_port }),
        };
        Self {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            description: Some(value.description.into()),
            model: Some(model),
        }
    }
}

impl TryFrom<AccessoryDescriptor> for crate::topology::AccessoryDescriptor {
    type Error = ConversionError;

    fn try_from(value: AccessoryDescriptor) -> Result<Self, Self::Error> {
        type ErrorBuilder =
            ConversionErrorBuilder<AccessoryDescriptor, crate::topology::AccessoryDescriptor>;

        let accessory_id: crate::topology::AccessoryId = value
            .id
            .ok_or(ErrorBuilder::field_not_set("id"))?
            .try_into()?;
        let accessory_name: crate::topology::AccessoryName = value
            .name
            .ok_or(ErrorBuilder::field_not_set("name"))?
            .try_into()?;
        let accessory_description: Option<crate::topology::AccessoryDescription> = value
            .description
            .map(TryFrom::try_from)
            .transpose()?;
        let accessory_model = match value.model
            .ok_or(ErrorBuilder::field_not_set("model"))? {
                accessory_descriptor::Model::MansonHcs3304(model_params) => AccessoryModel::MansonHcs3304 { 
                    serial_port: model_params.serial_port
                },
            };

        Ok(Self {
            id: accessory_id,
            name: accessory_name,
            description: accessory_description,
            model: accessory_model,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::proto::util::{network_interface_descriptor, EthernetInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};
    use crate::topology::{AccessoryDescriptor, AccessoryModel, AccessoryName, AccessoryDescription};
    use crate::topology::{DeviceDescription, DeviceName, DeviceTag};
    use crate::util::net::NetworkInterfaceConfiguration;
    use googletest::prelude::*;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn A_Topology_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {
        let device_id_1 = Uuid::new_v4();
        let device_id_2 = Uuid::new_v4();
        let accessory_id_1 = Uuid::new_v4();
        let accessory_id_2 = Uuid::new_v4();

        let native = crate::topology::Topology {
            devices: vec![
                crate::topology::DeviceDescriptor {
                    id: Clone::clone(&device_id_1).into(),
                    name: DeviceName::try_from("device-1")?,
                    description: DeviceDescription::try_from("Some device").ok(),
                    interface: crate::util::net::NetworkInterfaceDescriptor { 
                        name: crate::util::net::NetworkInterfaceName::try_from("tap0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                    tags: vec![DeviceTag::try_from("tag-1")?, DeviceTag::try_from("tag-2")?],
                    accessories: vec![AccessoryDescriptor { 
                        id: Clone::clone(&accessory_id_1).into(), 
                        name: AccessoryName::try_from("accessory-1")?, 
                        description: Some(AccessoryDescription::try_from("Some accessory")?), 
                        model: AccessoryModel::MansonHcs3304 { serial_port: "ttyS0".to_string() } 
                    }],
                },
                crate::topology::DeviceDescriptor {
                    id: Clone::clone(&device_id_2).into(),
                    name: DeviceName::try_from("device-2")?,
                    description: DeviceDescription::try_from("Some other device").ok(),
                    interface: crate::util::net::NetworkInterfaceDescriptor { 
                        name: crate::util::net::NetworkInterfaceName::try_from("tap1")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                    tags: vec![DeviceTag::try_from("tag-2")?],
                    accessories: vec![AccessoryDescriptor { 
                        id: Clone::clone(&accessory_id_2).into(), 
                        name: AccessoryName::try_from("accessory-2")?, 
                        description: Some(AccessoryDescription::try_from("Some other accessory")?), 
                        model: AccessoryModel::MansonHcs3304 { serial_port: "ttyS1".to_string() } 
                    }],
                },
            ],
        };

        let proto = Topology {
            device_descriptors: vec![
                DeviceDescriptor {
                    id: Some(Clone::clone(&device_id_1).into()),
                    name: Some(crate::proto::topology::DeviceName {
                        value: String::from("device-1"),
                    }),
                    description: Some(crate::proto::topology::DeviceDescription {
                        value: String::from("Some device"),
                    }),
                    interface: Some(NetworkInterfaceDescriptor{
                        name: Some(NetworkInterfaceName {
                            name: String::from("tap0"),
                        }),
                        configuration: Some(network_interface_descriptor::Configuration::Ethernet(
                            EthernetInterfaceConfiguration{},
                        )),
                    }),
                    tags: vec![
                        Some(crate::proto::topology::DeviceTag {
                            value: String::from("tag-1"),
                        })
                        .unwrap(),
                        Some(crate::proto::topology::DeviceTag {
                            value: String::from("tag-2"),
                        })
                        .unwrap(),
                    ],
                    accessories: vec![
                        crate::proto::topology::AccessoryDescriptor { 
                            id: Some(Clone::clone(&accessory_id_1).into()), 
                            name: Some(crate::proto::topology::AccessoryName {
                                value: String::from("accessory-1"),
                            }), 
                            description: Some(crate::proto::topology::AccessoryDescription {
                                value: String::from("Some accessory"),
                            }), 
                            model: Some(Model::MansonHcs3304(MansonHcs3304 { serial_port: "ttyS0".to_string() })), 
                        }
                    ]
                },
                DeviceDescriptor {
                    id: Some(Clone::clone(&device_id_2).into()),
                    name: Some(crate::proto::topology::DeviceName {
                        value: String::from("device-2"),
                    }),
                    description: Some(crate::proto::topology::DeviceDescription {
                        value: String::from("Some other device"),
                    }),
                    interface: Some(NetworkInterfaceDescriptor{
                        name: Some(NetworkInterfaceName {
                            name: String::from("tap1"),
                        }),
                        configuration: Some(network_interface_descriptor::Configuration::Ethernet(
                            EthernetInterfaceConfiguration{},
                        )),
                    }),
                    tags: vec![Some(crate::proto::topology::DeviceTag {
                        value: String::from("tag-2"),
                    })
                    .unwrap()],
                    accessories: vec![
                        crate::proto::topology::AccessoryDescriptor { 
                            id: Some(Clone::clone(&accessory_id_2).into()), 
                            name: Some(crate::proto::topology::AccessoryName {
                                value: String::from("accessory-2"),
                            }), 
                            description: Some(crate::proto::topology::AccessoryDescription {
                                value: String::from("Some other accessory"),
                            }), 
                            model: Some(Model::MansonHcs3304(MansonHcs3304 { serial_port: "ttyS1".to_string() })), 
                        }
                    ]
                },
            ],
        };

        verify_that!(
            native,
            eq(crate::topology::Topology::try_from(Clone::clone(&proto))?)
        )?;

        verify_that!(proto, eq(Topology::try_from(native)?))?;

        Ok(())
    }

    #[test]
    fn Converting_a_proto_Topology_to_a_native_Topology_should_fail_if_the_id_of_a_device_is_not_set(
    ) -> Result<()> {
        let proto = Topology {
            device_descriptors: vec![DeviceDescriptor {
                id: None,
                name: Some(crate::proto::topology::DeviceName {
                    value: String::from("device-1"),
                }),
                description: Some(crate::proto::topology::DeviceDescription {
                    value: String::from("Some device"),
                }),
                interface: Some(NetworkInterfaceDescriptor{
                    name: Some(NetworkInterfaceName {
                        name: String::from("tap0"),
                    }),
                    configuration: Some(network_interface_descriptor::Configuration::Ethernet(
                        EthernetInterfaceConfiguration{},
                    )),
                }),
                tags: vec![
                    Some(crate::proto::topology::DeviceTag {
                        value: String::from("tag-1"),
                    })
                    .unwrap(),
                    Some(crate::proto::topology::DeviceTag {
                        value: String::from("tag-2"),
                    })
                    .unwrap(),
                ],
                accessories: vec![],
            }],
        };

        let result = crate::topology::Topology::try_from(proto);

        verify_that!(
            result,
            err(eq(ConversionError::new::<
                DeviceDescriptor,
                crate::topology::DeviceDescriptor,
            >("Field 'id' not set")))
        )?;

        Ok(())
    }
}
