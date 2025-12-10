use leptos::prelude::*;
use opendut_lea_components::UserInputValue;
use opendut_model::peer::executor::container::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine};
use opendut_model::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind, ResultsUrl};
use crate::peers::configurator::types::PeerMisconfigurationError;

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
