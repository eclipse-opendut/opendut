use leptos::prelude::*;
use opendut_lea_components::UserInputValue;
use opendut_model::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_model::peer::executor::{ExecutorDescriptor, ExecutorDescriptors};
use opendut_model::topology::{DeviceDescriptor, Topology};
use opendut_model::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName};
use crate::peers::configurator::types::devices::{DeviceMisconfigurationError, UserDeviceConfiguration};
use crate::peers::configurator::types::executor::UserPeerExecutor;
use crate::peers::configurator::types::network::UserPeerNetwork;

pub mod devices;
pub mod executor;
pub mod general;
pub mod network;

pub const EMPTY_DEVICE_NAME_ERROR_MESSAGE: &str = "Enter a valid device name.";
pub const EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE: &str = "Enter a valid container image.";

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)] // "all variants have the same prefix: `Invalid`"
pub enum PeerMisconfigurationError {
    #[error("Invalid peer name")]
    InvalidPeerName,
    #[error("{0}")]
    InvalidDevice(DeviceMisconfigurationError),
    #[error("Invalid peer network descriptor")]
    InvalidPeerNetwork,
    #[error("Invalid peer executor")]
    InvalidPeerExecutor,
}

#[derive(Clone, Debug)]
pub struct UserPeerConfiguration {
    pub id: PeerId,
    pub name: UserInputValue,
    pub location: UserInputValue,
    pub devices: Vec<RwSignal<UserDeviceConfiguration>>,
    pub network: UserPeerNetwork,
    pub executors: Vec<RwSignal<UserPeerExecutor>>,
    pub is_new: bool,
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
        let bridge_name = configuration.network
            .bridge_name
            .right_ok_or(PeerMisconfigurationError::InvalidPeerNetwork)
            .and_then(|bridge_name| {
                if bridge_name.is_empty() {
                    Ok(None)
                } else {
                    match NetworkInterfaceName::try_from(bridge_name) {
                        Ok(name) => Ok(Some(name)),
                        Err(_) => Err(PeerMisconfigurationError::InvalidPeerNetwork)
                    }
                }
            })?;
        let network_interfaces = configuration.network
            .network_interfaces
            .into_iter()
            .map(|signal| signal.get_untracked())
            .map(|interface| {
                NetworkInterfaceDescriptor::try_from(interface)
                    .map_err(|_|  PeerMisconfigurationError::InvalidPeerNetwork)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let devices = configuration
            .devices
            .into_iter()
            .map(|signal| signal.get_untracked())
            .map(|configuration| {
                DeviceDescriptor::try_from(configuration)
                    .map_err(PeerMisconfigurationError::InvalidDevice)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let executors = configuration
            .executors
            .into_iter()
            .map(|signal| signal.get_untracked())
            .map(|executor| {
                ExecutorDescriptor::try_from(executor)
                    .map_err(|_|  PeerMisconfigurationError::InvalidPeerExecutor)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(PeerDescriptor {
            id: configuration.id,
            name,
            location: Some(location),
            network: PeerNetworkDescriptor::new(network_interfaces, bridge_name),
            topology: Topology::new(devices),
            executors: ExecutorDescriptors {
                executors
            },
        })
    }
}