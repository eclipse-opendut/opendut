use leptos::{RwSignal, SignalGetUntracked};

use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkConfiguration};
use opendut_types::peer::executor::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine, ExecutorDescriptor, ExecutorDescriptors};
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

use crate::components::UserInputValue;

pub const EMPTY_DEVICE_NAME_ERROR_MESSAGE: &str = "Enter a valid device name.";
pub const EMPTY_DEVICE_INTERFACE_ERROR_MESSAGE: &str = "Enter a valid interface name!";
pub const EMPTY_CONTAINER_IMAGE_ERROR_MESSAGE: &str = "Enter a valid container image.";

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)] // "all variants have the same prefix: `Invalid`"
pub enum PeerMisconfigurationError {
    #[error("Invalid peer name")]
    InvalidPeerName,
    #[error("{0}")]
    InvalidDevice(DeviceMisconfigurationError),
    #[error("Invalid network configuration")]
    InvalidPeerNetworkConfiguration,
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
    pub network_interfaces: Vec<RwSignal<UserPeerNetworkInterface>>,
    pub executors: Vec<RwSignal<UserPeerExecutor>>,
    pub is_new: bool,
}

#[derive(Clone, Debug)]
pub enum UserPeerExecutor {
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
        is_collapsed: bool,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserContainerEnv {
    pub name: UserInputValue,
    pub value: UserInputValue
}


#[derive(Clone, Debug)]
pub struct UserPeerNetworkInterface {
    pub name: NetworkInterfaceName,
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
        let network_interfaces = configuration
            .network_interfaces
            .into_iter()
            .map(|signal| signal.get_untracked())
            .map(|interface| {
                NetworkInterfaceDescriptor::try_from(interface)
                    .map_err(|_|  PeerMisconfigurationError::InvalidPeerNetworkConfiguration)
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
            network_configuration: PeerNetworkConfiguration::new(network_interfaces),
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
            interface: NetworkInterfaceDescriptor {
                name: interface,
                configuration: NetworkInterfaceConfiguration::Ethernet, // TODO: Do not assume Ethernet here
            },
            tags: vec![],
        })
    }
}

impl TryFrom<UserPeerNetworkInterface> for NetworkInterfaceDescriptor {
    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserPeerNetworkInterface) -> Result<Self, Self::Error> {
        Ok(Self {
            name: configuration.name,
            configuration: NetworkInterfaceConfiguration::Ethernet, // TODO: Do not assume Ethernet here
        })
    }
}

impl TryFrom<UserPeerExecutor> for ExecutorDescriptor {
    type Error = PeerMisconfigurationError;

    fn try_from(configuration: UserPeerExecutor) -> Result<Self, Self::Error> {
        match configuration {
            UserPeerExecutor::Container {
                engine,
                name,
                image,
                volumes,
                devices,
                envs,
                ports,
                command,
                args,
                ..
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
                Ok(ExecutorDescriptor::Container {
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
        }
        
    }
}

