use crate::peers::configurator::types::network::UserNetworkInterface;
use leptos::prelude::*;
use opendut_lea_components::UserInputValue;
use opendut_model::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_model::topology::{DeviceDescriptor, Topology};
use opendut_model::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName};
use crate::peers::configurator::types::devices::{DeviceMisconfigurationError, UserDeviceConfiguration};
use crate::peers::configurator::types::network::UserPeerNetwork;

pub mod devices;
pub mod network;
pub mod validation;

pub const EMPTY_DEVICE_NAME_ERROR_MESSAGE: &str = "Enter a valid device name.";

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)] // "all variants have the same prefix: `Invalid`"
pub enum PeerMisconfigurationError {
    #[error("Invalid peer name")]
    InvalidPeerName,
    #[error("{0}")]
    InvalidDevice(DeviceMisconfigurationError),
    #[error("Invalid peer network descriptor")]
    InvalidPeerNetwork,
}

#[derive(Clone, Debug)]
pub struct UserPeerDescriptor {
    pub id: PeerId,
    pub name: UserInputValue,
    pub location: UserInputValue,
    pub devices: Vec<RwSignal<UserDeviceConfiguration>>,
    pub network: UserPeerNetwork,
    pub is_new: bool,
}

impl TryFrom<UserPeerDescriptor> for PeerDescriptor {
    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserPeerDescriptor) -> Result<Self, Self::Error> {
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

        Ok(PeerDescriptor {
            id: configuration.id,
            name,
            location: Some(location),
            network: PeerNetworkDescriptor::new(network_interfaces, bridge_name),
            topology: Topology::new(devices),
        })
    }
}

impl UserPeerDescriptor {
    pub fn apply_descriptor(
        &mut self,
        descriptor: PeerDescriptor,
        clusters: Vec<opendut_model::cluster::ClusterDescriptor>,
    ) {
        self.name = UserInputValue::Right(descriptor.name.value().to_owned());
        self.is_new = false;
        self.location = UserInputValue::Right(descriptor.location.unwrap_or_default().value());
        self.devices = descriptor
            .topology
            .devices
            .into_iter()
            .map(|device| {
                let mut configured_clusters = vec![];

                for cluster in &clusters {
                    for device_id in &cluster.devices {
                        if device.id == *device_id {
                            configured_clusters.push(cluster.clone());
                        }
                    }
                }
                RwSignal::new(UserDeviceConfiguration {
                    id: device.id,
                    name: UserInputValue::Right(device.name.to_string()),
                    interface: Some(device.interface),
                    description: UserInputValue::Right(
                        device.description.unwrap_or_default().to_string(),
                    ),
                    is_collapsed: true,
                    contained_in_clusters: configured_clusters,
                    tags: device.tags,
                })
            })
            .collect::<Vec<_>>();
        if let Some(bridge_name) = descriptor.network.bridge_name {
            self.network.bridge_name = UserInputValue::Right(bridge_name.name());
        }
        self.network.network_interfaces = descriptor
            .network
            .interfaces
            .into_iter()
            .map(|interface| RwSignal::new(UserNetworkInterface::from(interface)))
            .collect();
    }
}
