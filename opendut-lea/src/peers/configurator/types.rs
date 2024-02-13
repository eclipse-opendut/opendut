use leptos::{RwSignal, SignalGetUntracked};

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName};
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
use opendut_types::util::net::NetworkInterfaceName;

use crate::components::UserInputValue;

pub const EMPTY_DEVICE_NAME_ERROR_MESSAGE: &str = "Enter a valid device name.";
pub const EMPTY_DEVICE_INTERFACE_ERROR_MESSAGE: &str = "Enter a valid interface name!";

#[derive(thiserror::Error, Clone, Debug)]
pub enum PeerMisconfigurationError {
    #[error("Invalid peer name")]
    InvalidPeerName,
    #[error("{0}")]
    InvalidDevice(DeviceMisconfigurationError),
}

#[derive(thiserror::Error, Clone, Debug)]
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
pub struct UserPeerConfiguration {
    pub id: PeerId,
    pub name: UserInputValue,
    pub location: UserInputValue,
    pub devices: Vec<RwSignal<UserDeviceConfiguration>>,
    pub is_new: bool,
}

#[derive(Clone, Debug)]
pub struct UserDeviceConfiguration {
    pub id: DeviceId,
    pub name: UserInputValue,
    pub description: UserInputValue,
    pub interface: UserInputValue,
    pub is_collapsed: bool,
}

impl TryFrom<UserPeerConfiguration> for PeerDescriptor {
    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserPeerConfiguration) -> Result<Self, Self::Error> {
        let name = configuration
            .name
            .right_ok_or(PeerMisconfigurationError::InvalidPeerName)
            .and_then(|name| {
                PeerName::try_from(name).map_err(|_| PeerMisconfigurationError::InvalidPeerName)
            })?;
        let location = configuration
            .location
            .right_ok_or(PeerMisconfigurationError::InvalidPeerName)
            .and_then(|location| {
                PeerLocation::try_from(location)
                    .map_err(|_| PeerMisconfigurationError::InvalidPeerName)
            })?;
        let devices = configuration
            .devices
            .into_iter()
            .map(|signal| signal.get_untracked())
            .map(|configuration| {
                DeviceDescriptor::try_from(configuration)
                    .map_err(PeerMisconfigurationError::InvalidDevice)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PeerDescriptor {
            id: configuration.id,
            name,
            location: Some(location),
            topology: Topology::new(devices),
        })
    }
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
        let interface = configuration
            .interface
            .right_ok_or(DeviceMisconfigurationError::InvalidDeviceInterface)
            .and_then(|interface_name| {
                NetworkInterfaceName::try_from(interface_name)
                    .map_err(|_| DeviceMisconfigurationError::InvalidDeviceInterface)
            })?;
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
