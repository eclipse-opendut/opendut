use crate::proto::{conversion, ConversionError, ConversionErrorBuilder, ConversionResult};

include!(concat!(env!("OUT_DIR"), "/opendut.types.topology.rs"));


conversion! {
    type Model = crate::topology::DeviceId;
    type Proto = DeviceId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.0.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| crate::topology::DeviceId(uuid.into()))
    }
}
impl From<uuid::Uuid> for DeviceId {
    fn from(value: uuid::Uuid) -> Self {
        Self {
            uuid: Some(value.into()),
        }
    }
}

conversion! {
    type Model = crate::topology::Topology;
    type Proto = Topology;

    fn from(value: Model) -> Proto {
        Proto {
            device_descriptors: value
                .devices
                .into_iter()
                .map(DeviceDescriptor::from)
                .collect(),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        value
            .device_descriptors
            .into_iter()
            .map(DeviceDescriptor::try_into)
            .collect::<Result<_, _>>()
            .map(|devices| Model { devices })
    }
}

conversion! {
    type Model = crate::topology::DeviceName;
    type Proto = DeviceName;

    fn from(value: Model) -> Proto {
        Proto {
            value: String::from(value.value()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
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
impl TryFrom<DeviceDescription> for crate::topology::DeviceDescription {
    type Error = ConversionError;

    fn try_from(value: DeviceDescription) -> Result<Self, Self::Error> {
        type ErrorBuilder =
            ConversionErrorBuilder<DeviceDescription, crate::topology::DeviceDescription>;

        crate::topology::DeviceDescription::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::topology::DeviceTag;
    type Proto = DeviceTag;

    fn from(value: Model) -> Proto {
        Proto {
            value: String::from(value.value()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::topology::DeviceDescriptor;
    type Proto = DeviceDescriptor;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            description: Some(value.description.into()),
            interface: Some(value.interface.into()),
            tags: value.tags.into_iter().map(|value| value.into()).collect(),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let device_id: crate::topology::DeviceId = extract!(value.id)?.try_into()?;

        let device_name: crate::topology::DeviceName = extract!(value.name)?.try_into()?;

        let device_description: Option<crate::topology::DeviceDescription> =
            value.description.map(TryFrom::try_from).transpose()?;

        let interface: crate::util::net::NetworkInterfaceId = extract!(value.interface)?.try_into()?;

        let device_tags: Vec<crate::topology::DeviceTag> = value.tags
            .into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<_, _>>()?;

        Ok(Model {
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
    use crate::proto::util::NetworkInterfaceId;
    use crate::topology::{DeviceDescription, DeviceName, DeviceTag};
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
                    interface: network_interface_id1.into(),
                    tags: vec![DeviceTag::try_from("tag-1")?, DeviceTag::try_from("tag-2")?],
                },
                crate::topology::DeviceDescriptor {
                    id: Clone::clone(&device_id_2).into(),
                    name: DeviceName::try_from("device-2")?,
                    description: DeviceDescription::try_from("Some other device").ok(),
                    interface: network_interface_id2.into(),
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
                    interface: Some(NetworkInterfaceId {
                        uuid: Some(network_interface_id1.clone().into()),
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
                    interface: Some(NetworkInterfaceId {
                        uuid: Some(network_interface_id2.clone().into()),
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
            eq(&crate::topology::Topology::try_from(Clone::clone(&proto))?)
        )?;

        verify_that!(proto, eq(&Topology::try_from(native)?))?;

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
                interface: Some(NetworkInterfaceId {
                    uuid: Some(Uuid::new_v4().into()),
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
            err(eq(&ConversionError::new::<
                DeviceDescriptor,
                crate::topology::DeviceDescriptor,
            >("Field 'id' not set")))
        )?;

        Ok(())
    }
}
