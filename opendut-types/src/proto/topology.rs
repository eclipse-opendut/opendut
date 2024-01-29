use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.topology.rs"));

impl From<crate::topology::DeviceId> for DeviceId {
    fn from(value: crate::topology::DeviceId) -> Self {
        Self {
            uuid: Some(value.0.into())
        }
    }
}

impl TryFrom<DeviceId> for crate::topology::DeviceId {
    type Error = ConversionError;

    fn try_from(value: DeviceId) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeviceId, crate::topology::DeviceId>;

        value.uuid
            .ok_or(ErrorBuilder::new("Uuid not set"))
            .map(|uuid| Self(uuid.into()))
    }
}

impl From<uuid::Uuid> for DeviceId {
    fn from(value: uuid::Uuid) -> Self {
        Self {
            uuid: Some(value.into())
        }
    }
}

impl From<crate::topology::Topology> for Topology {
    fn from(value: crate::topology::Topology) -> Self {
        Self {
            device_descriptors: value.devices
                .into_iter()
                .map(DeviceDescriptor::from)
                .collect(),
        }
    }
}

impl TryFrom<Topology> for crate::topology::Topology {
    type Error = ConversionError;

    fn try_from(value: Topology) -> Result<Self, Self::Error> {
        value.device_descriptors
            .into_iter()
            .map(DeviceDescriptor::try_into)
            .collect::<Result<_, _>>()
            .map(|devices| Self {
                devices
            })
    }
}

impl From<crate::topology::DeviceDescriptor> for DeviceDescriptor {
    fn from(value: crate::topology::DeviceDescriptor) -> Self {
        Self {
            id: Some(value.id.into()),
            name: value.name,
            description: value.description,
            interface: Some(value.interface.into()),
            tags: value.tags,
        }
    }
}

impl TryFrom<DeviceDescriptor> for crate::topology::DeviceDescriptor {
    type Error = ConversionError;

    fn try_from(value: DeviceDescriptor) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<DeviceDescriptor, crate::topology::DeviceDescriptor>;

        let device_id: crate::topology::DeviceId = value.id
            .ok_or(ErrorBuilder::new("Id not set"))?
            .try_into()?;
        let interface: crate::util::net::NetworkInterfaceName = value.interface
            .ok_or(ErrorBuilder::new("Interface not set"))?
            .try_into()?;

        Ok(Self {
            id: device_id,
            name: value.name,
            description: value.description,
            interface,
            tags: value.tags,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use googletest::prelude::*;
    use uuid::Uuid;
    use crate::proto::util::NetworkInterfaceName;

    use super::*;

    #[test]
    fn A_Topology_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

        let device_id_1 = Uuid::new_v4();
        let device_id_2 = Uuid::new_v4();

        let native = crate::topology::Topology {
            devices: vec![
                crate::topology::DeviceDescriptor {
                    id: Clone::clone(&device_id_1).into(),
                    name: String::from("device-1"),
                    description: String::from("Some device"),
                    interface: crate::util::net::NetworkInterfaceName::try_from("tap0").unwrap(),
                    tags: vec![String::from("tag-1"), String::from("tag-2")],
                },
                crate::topology::DeviceDescriptor {
                    id: Clone::clone(&device_id_2).into(),
                    name: String::from("device-2"),
                    description: String::from("Some other device"),
                    interface: crate::util::net::NetworkInterfaceName::try_from("tap1").unwrap(),
                    tags: vec![String::from("tag-2")],
                }
            ],
        };

        let proto = Topology {
            device_descriptors: vec![
                DeviceDescriptor {
                    id: Some(Clone::clone(&device_id_1).into()),
                    name: String::from("device-1"),
                    description: String::from("Some device"),
                    interface: Some(NetworkInterfaceName { name: String::from("tap0") }),
                    tags: vec![String::from("tag-1"), String::from("tag-2")],
                },
                DeviceDescriptor {
                    id: Some(Clone::clone(&device_id_2).into()),
                    name: String::from("device-2"),
                    description: String::from("Some other device"),
                    interface: Some(NetworkInterfaceName { name: String::from("tap1") }),
                    tags: vec![String::from("tag-2")],
                }
            ],
        };

        verify_that!(native, eq(crate::topology::Topology::try_from(Clone::clone(&proto))?))?;

        verify_that!(proto, eq(Topology::try_from(native)?))?;

        Ok(())
    }

    #[test]
    fn Converting_a_proto_Topology_to_a_native_Topology_should_fail_if_the_id_of_a_device_is_not_set() -> Result<()> {

        let proto = Topology {
            device_descriptors: vec![
                DeviceDescriptor {
                    id: None,
                    name: String::from("device-1"),
                    description: String::from("Some device"),
                    interface: Some(NetworkInterfaceName { name: String::from("tap0") }),
                    tags: vec![String::from("tag-1"), String::from("tag-2")],
                },
            ],
        };

        let result = crate::topology::Topology::try_from(proto);

        verify_that!(result, err(eq(ConversionError::new::<DeviceDescriptor, crate::topology::DeviceDescriptor>("Id not set"))))?;

        Ok(())
    }
}
