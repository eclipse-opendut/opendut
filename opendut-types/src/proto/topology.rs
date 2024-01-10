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
            devices: value.devices
                .into_iter()
                .map(Device::from)
                .collect(),
        }
    }
}

impl TryFrom<Topology> for crate::topology::Topology {
    type Error = ConversionError;

    fn try_from(value: Topology) -> Result<Self, Self::Error> {
        value.devices
            .into_iter()
            .map(Device::try_into)
            .collect::<Result<_, _>>()
            .map(|devices| Self {
                devices
            })
    }
}

impl From<crate::topology::Device> for Device {
    fn from(value: crate::topology::Device) -> Self {
        Self {
            id: Some(value.id.into()),
            name: value.name,
            description: value.description,
            location: value.location,
            interface: Some(value.interface.into()),
            tags: value.tags,
        }
    }
}

impl TryFrom<Device> for crate::topology::Device {
    type Error = ConversionError;

    fn try_from(value: Device) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<Device, crate::topology::Device>;

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
            location: value.location,
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
                crate::topology::Device {
                    id: Clone::clone(&device_id_1).into(),
                    name: String::from("device-1"),
                    description: String::from("Some device"),
                    location: String::from("Ulm"),
                    interface: crate::util::net::NetworkInterfaceName::try_from("tap0").unwrap(),
                    tags: vec![String::from("tag-1"), String::from("tag-2")],
                },
                crate::topology::Device {
                    id: Clone::clone(&device_id_2).into(),
                    name: String::from("device-2"),
                    description: String::from("Some other device"),
                    location: String::from("Stuttgart"),
                    interface: crate::util::net::NetworkInterfaceName::try_from("tap1").unwrap(),
                    tags: vec![String::from("tag-2")],
                }
            ],
        };

        let proto = Topology {
            devices: vec![
                Device {
                    id: Some(Clone::clone(&device_id_1).into()),
                    name: String::from("device-1"),
                    description: String::from("Some device"),
                    location: String::from("Ulm"),
                    interface: Some(NetworkInterfaceName { name: String::from("tap0") }),
                    tags: vec![String::from("tag-1"), String::from("tag-2")],
                },
                Device {
                    id: Some(Clone::clone(&device_id_2).into()),
                    name: String::from("device-2"),
                    description: String::from("Some other device"),
                    location: String::from("Stuttgart"),
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
            devices: vec![
                Device {
                    id: None,
                    name: String::from("device-1"),
                    description: String::from("Some device"),
                    location: String::from("Ulm"),
                    interface: Some(NetworkInterfaceName { name: String::from("tap0") }),
                    tags: vec![String::from("tag-1"), String::from("tag-2")],
                },
            ],
        };

        let result = crate::topology::Topology::try_from(proto);

        verify_that!(result, err(eq(ConversionError::new::<Device, crate::topology::Device>("Id not set"))))?;

        Ok(())
    }
}
