use opendut_lea_components::UserInputValue;
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName};
use opendut_model::util::net::NetworkInterfaceId;

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)] // "all variants have the same prefix: `Invalid`"
pub enum DeviceMisconfigurationError {
    #[error("Invalid device name")]
    InvalidDeviceName,
    #[error("Invalid device location")]
    InvalidDeviceLocation,
    #[error("Invalid device interface")]
    InvalidDeviceInterface,
    #[error("Invalid device description")]
    InvalidDeviceDescription,
}

#[derive(Clone, Debug)]
pub struct UserDeviceConfiguration {
    pub id: DeviceId,
    pub name: UserInputValue,
    pub description: UserInputValue,
    pub interface: Option<NetworkInterfaceId>,
    pub is_collapsed: bool,
    pub contained_in_clusters: Vec<ClusterDescriptor>,
}

impl TryFrom<UserDeviceConfiguration> for DeviceDescriptor {
    type Error = DeviceMisconfigurationError;

    fn try_from(configuration: UserDeviceConfiguration) -> Result<Self, Self::Error> {
        let name = configuration
            .name
            .right_ok_or(DeviceMisconfigurationError::InvalidDeviceName)
            .and_then(|name| {
                DeviceName::try_from(name)
                    .map_err(|_| DeviceMisconfigurationError::InvalidDeviceName)
            })?;

        let interface = configuration.interface.unwrap();
        let description = configuration
            .description
            .right_ok_or(DeviceMisconfigurationError::InvalidDeviceDescription)
            .and_then(|description| {
                DeviceDescription::try_from(description)
                    .map_err(|_| DeviceMisconfigurationError::InvalidDeviceDescription)
            })?;
        Ok(DeviceDescriptor {
            id: configuration.id,
            name,
            description: Some(description),
            interface,
            tags: vec![],
        })
    }
}
