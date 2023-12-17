use leptos::{RwSignal, SignalGetUntracked};

use opendut_types::peer::{PeerDescriptor, PeerId, PeerName};
use opendut_types::topology::{Device, DeviceId, InterfaceName, Topology};

use crate::components::UserInputValue;

pub const EMPTY_DEVICE_NAME_ERROR_MESSAGE: &'static str = "The name of a device may not be empty!";
pub const EMPTY_DEVICE_INTERFACE_ERROR_MESSAGE: &'static str = "Enter a valid interface name!";

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
}

#[derive(Clone, Debug)]
pub struct UserPeerConfiguration {
    pub id: PeerId,
    pub name: UserInputValue,
    pub devices: Vec<RwSignal<UserDeviceConfiguration>>,
    pub is_new: bool,
}

#[derive(Clone, Debug)]
pub struct UserDeviceConfiguration {
    pub id: DeviceId,
    pub name: UserInputValue,
    pub description: String,
    pub location: UserInputValue,
    pub interface: UserInputValue,
    pub is_collapsed: bool,
}

impl TryFrom<UserPeerConfiguration> for PeerDescriptor {

    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserPeerConfiguration) -> Result<Self, Self::Error> {
        let name = configuration.name
            .right_ok_or(PeerMisconfigurationError::InvalidPeerName)
            .and_then(|name| PeerName::try_from(name)
                .map_err(|_| PeerMisconfigurationError::InvalidPeerName))?;
        let devices = configuration.devices.into_iter()
            .map(|signal| signal.get_untracked())
            .map(|configuration| Device::try_from(configuration)
                .map_err(PeerMisconfigurationError::InvalidDevice)
            )
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PeerDescriptor {
            id: configuration.id,
            name,
            topology: Topology::new(devices),
        })
    }
}

impl TryFrom<UserDeviceConfiguration> for Device {
    type Error = DeviceMisconfigurationError;

    fn try_from(configuration: UserDeviceConfiguration) -> Result<Self, Self::Error> {
        let name = configuration.name
            .right_ok_or(DeviceMisconfigurationError::InvalidDeviceName)?;
        let location = configuration.location
            .right_ok_or(DeviceMisconfigurationError::InvalidDeviceLocation)?;
        let interface = configuration.interface
            .right_ok_or(DeviceMisconfigurationError::InvalidDeviceInterface)
            .and_then(|interface_name| InterfaceName::try_from(interface_name)
                .map_err(|_| DeviceMisconfigurationError::InvalidDeviceInterface)
            )?;
        Ok(Device{
            id: configuration.id,
            name,
            description: configuration.description,
            location,
            interface,
            tags: vec![],
        })
    }
}
