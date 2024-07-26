use crate::proto::{ConversionError, ConversionErrorBuilder};

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

        Ok(Self {
            id: device_id,
            name: device_name,
            description: device_description,
            interface,
            tags: device_tags,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use crate::proto::util::{network_interface_descriptor, EthernetInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName, NetworkInterfaceId};
    use crate::topology::{DeviceDescription, DeviceName, DeviceTag};
    use crate::util::net::NetworkInterfaceConfiguration;
    use googletest::prelude::*;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn A_Topology_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {
        let device_id_1 = Uuid::new_v4();
        let device_id_2 = Uuid::new_v4();
        let network_interface_id1 = Uuid::new_v4();
        let network_interface_id2 = Uuid::new_v4();

        let native = crate::topology::Topology {
            devices: vec![
                crate::topology::DeviceDescriptor {
                    id: Clone::clone(&device_id_1).into(),
                    name: DeviceName::try_from("device-1")?,
                    description: DeviceDescription::try_from("Some device").ok(),
                    interface: crate::util::net::NetworkInterfaceDescriptor {
                        id: network_interface_id1.into(),
                        name: crate::util::net::NetworkInterfaceName::try_from("tap0")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                    tags: vec![DeviceTag::try_from("tag-1")?, DeviceTag::try_from("tag-2")?],
                },
                crate::topology::DeviceDescriptor {
                    id: Clone::clone(&device_id_2).into(),
                    name: DeviceName::try_from("device-2")?,
                    description: DeviceDescription::try_from("Some other device").ok(),
                    interface: crate::util::net::NetworkInterfaceDescriptor {
                        id: network_interface_id2.into(),
                        name: crate::util::net::NetworkInterfaceName::try_from("tap1")?,
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                    tags: vec![DeviceTag::try_from("tag-2")?],
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
                    interface: Some(NetworkInterfaceDescriptor {
                        id: Some(NetworkInterfaceId {
                            uuid: Some(network_interface_id1.clone().into()),
                        }),
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
                },
                DeviceDescriptor {
                    id: Some(Clone::clone(&device_id_2).into()),
                    name: Some(crate::proto::topology::DeviceName {
                        value: String::from("device-2"),
                    }),
                    description: Some(crate::proto::topology::DeviceDescription {
                        value: String::from("Some other device"),
                    }),
                    interface: Some(NetworkInterfaceDescriptor {
                        id: Some(NetworkInterfaceId {
                            uuid: Some(network_interface_id2.clone().into()),
                        }),
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
    fn Converting_a_proto_Topology_to_a_native_Topology_should_fail_if_the_id_of_a_device_is_not_set() -> Result<()> {
        let proto = Topology {
            device_descriptors: vec![DeviceDescriptor {
                id: None,
                name: Some(crate::proto::topology::DeviceName {
                    value: String::from("device-1"),
                }),
                description: Some(crate::proto::topology::DeviceDescription {
                    value: String::from("Some device"),
                }),
                interface: Some(NetworkInterfaceDescriptor {
                    id: Some(NetworkInterfaceId {
                        uuid: Some(Uuid::new_v4().into()),
                    }),
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
