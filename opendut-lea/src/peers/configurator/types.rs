use leptos::{RwSignal, SignalGetUntracked};
use opendut_types::cluster::ClusterId;
use opendut_types::peer::executor::{ExecutorDescriptor, ExecutorId};
use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
use opendut_types::peer::executor::{container::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine}, ExecutorKind, ExecutorDescriptors, ResultsUrl};
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
use opendut_types::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

use crate::components::UserInputValue;
use crate::util::net::UserNetworkInterfaceConfiguration;

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
pub struct UserPeerConfiguration {
    pub id: PeerId,
    pub name: UserInputValue,
    pub location: UserInputValue,
    pub devices: Vec<RwSignal<UserDeviceConfiguration>>,
    pub network: UserPeerNetwork,
    pub executors: Vec<RwSignal<UserPeerExecutor>>,
    pub is_new: bool,
}

#[derive(Clone, Debug)]
pub struct UserPeerExecutor {
    pub id: ExecutorId,
    pub kind: UserPeerExecutorKind,
    pub results_url: UserInputValue,
    pub is_collapsed: bool,
}

#[derive(Clone, Debug)]
pub enum UserPeerExecutorKind {
    Container {
        engine: Engine,
        name: UserInputValue,
        image: UserInputValue,
        volumes: Vec<RwSignal<UserInputValue>>,
        devices: Vec<RwSignal<UserInputValue>>,
        envs: Vec<RwSignal<UserContainerEnv>>,
        ports: Vec<RwSignal<UserInputValue>>,
        command: UserInputValue,
        args: Vec<RwSignal<UserInputValue>>,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserContainerEnv {
    pub name: UserInputValue,
    pub value: UserInputValue
}

#[derive(Clone, Debug)]
pub struct UserPeerNetwork {
    pub network_interfaces: Vec<RwSignal<UserNetworkInterface>>,
    pub bridge_name: UserInputValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserNetworkInterface {
    pub id: NetworkInterfaceId,
    pub name: NetworkInterfaceName,
    pub configuration: UserNetworkInterfaceConfiguration,
}

#[derive(Clone, Debug)]
pub struct UserDeviceConfiguration {
    pub id: DeviceId,
    pub name: UserInputValue,
    pub description: UserInputValue,
    pub interface: Option<NetworkInterfaceId>,
    pub is_collapsed: bool,
    pub contained_in_clusters: Vec<ClusterId>
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

impl From<NetworkInterfaceDescriptor> for UserNetworkInterface {
    fn from(interface: NetworkInterfaceDescriptor) -> Self {
        Self {
            id: interface.id,
            name: interface.name,
            configuration: UserNetworkInterfaceConfiguration::from(interface.configuration),
        }
    }
}
impl TryFrom<UserNetworkInterface> for NetworkInterfaceDescriptor {
    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserNetworkInterface) -> Result<Self, Self::Error> {
        Ok(Self {
            id: configuration.id,
            name: configuration.name,
            configuration: configuration.configuration.inner,
        })
    }
}

impl TryFrom<UserPeerExecutor> for ExecutorDescriptor {
    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserPeerExecutor) -> Result<Self, Self::Error> {
        let UserPeerExecutor { id, kind, results_url, is_collapsed: _ } = configuration;

        let kind = match kind {
            UserPeerExecutorKind::Container {
                engine,
                name,
                image,
                volumes,
                devices,
                envs,
                ports,
                command,
                args,
            } => {
                let name = name
                    .right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                    .and_then(|name| {
                        ContainerName::try_from(name)
                            .map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor)
                    })?;
                let image = image
                    .right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                    .and_then(|description| {
                        ContainerImage::try_from(description)
                            .map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor)
                    })?;
                let command = command
                    .right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                    .and_then(|command| {
                        ContainerCommand::try_from(command)
                            .map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor)
                    })?;
                let volumes = volumes
                    .into_iter()
                    .map(|signal| signal.get_untracked())
                    .map(|volume| {
                        volume.right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                            .and_then(|volume| ContainerVolume::try_from(volume).map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let devices = devices
                    .into_iter()
                    .map(|signal| signal.get_untracked())
                    .map(|device| {
                        device.right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                            .and_then(|device| ContainerDevice::try_from(device).map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let ports = ports
                    .into_iter()
                    .map(|signal| signal.get_untracked())
                    .map(|port| {
                        port.right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                            .and_then(|port| ContainerPortSpec::try_from(port).map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let args = args
                    .into_iter()
                    .map(|signal| signal.get_untracked())
                    .map(|arg| {
                        arg.right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                            .and_then(|arg| ContainerCommandArgument::try_from(arg).map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let envs = envs
                    .into_iter()
                    .map(|signal| signal.get_untracked())
                    .map(|env| {
                        env.name.right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                            .and_then(|name| env.value.right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
                                .and_then(|value| ContainerEnvironmentVariable::new(name, value)
                                    .map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor)))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(ExecutorKind::Container {
                    engine,
                    name,
                    image,
                    volumes,
                    devices,
                    envs,
                    ports,
                    command,
                    args,
                })
            }
        }?;

        let results_url = results_url
            .right_ok_or(PeerMisconfigurationError::InvalidPeerExecutor)
            .and_then(|results_url| {
                if results_url.is_empty() {
                    Ok(None)
                } else {
                    Some(ResultsUrl::try_from(results_url)
                        .map_err(|_| PeerMisconfigurationError::InvalidPeerExecutor)).transpose()
                }
            })?;

        Ok(ExecutorDescriptor { id, kind, results_url })
    }
}

