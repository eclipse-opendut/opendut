use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.executor.rs"));

impl From<crate::peer::executor::ExecutorDescriptors> for ExecutorDescriptors {
    fn from(value: crate::peer::executor::ExecutorDescriptors) -> Self {
        Self {
            executors: value
                .executors
                .into_iter()
                .map(ExecutorDescriptor::from)
                .collect(),
        }
    }
}

impl TryFrom<ExecutorDescriptors> for crate::peer::executor::ExecutorDescriptors{
    type Error = ConversionError;

    fn try_from(value: ExecutorDescriptors) -> Result<Self, Self::Error> {
        value
            .executors
            .into_iter()
            .map(ExecutorDescriptor::try_into)
            .collect::<Result<_, _>>()
            .map(|executors| Self { executors} )
    }
}

impl From<crate::peer::executor::ExecutorDescriptor> for ExecutorDescriptor {
    fn from(value: crate::peer::executor::ExecutorDescriptor) -> Self {
        match value {
            crate::peer::executor::ExecutorDescriptor::Executable => {
                ExecutorDescriptor {
                    descriptor: Some(executor_descriptor::Descriptor::Executable(
                       Executable {}
                    ))
                }
            }
            crate::peer::executor::ExecutorDescriptor::Container { 
                engine,
                name, 
                image, 
                volumes, 
                devices, 
                envs,
                ports,
                command,
                args
            } => {
                ExecutorDescriptor { 
                    descriptor: Some(executor_descriptor::Descriptor::Container(
                        Container {
                            engine: Some(engine.into()),
                            name: Some(name.into()),
                            image: Some(image.into()),
                            volumes: volumes.into_iter().map(|volume| volume.into()).collect(),
                            devices: devices.into_iter().map(|device| device.into()).collect(),
                            envs: envs.into_iter().map(|env| env.into()).collect(),
                            ports: ports.into_iter().map(|port| port.into()).collect(),
                            command: Some(command.into()),
                            args: args.into_iter().map(|arg| arg.into()).collect(),
                        }
                    ))
                }
            }
        }
    }
}

impl TryFrom<ExecutorDescriptor> for crate::peer::executor::ExecutorDescriptor {
    type Error = ConversionError;

    fn try_from(value: ExecutorDescriptor) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ExecutorDescriptor, crate::peer::executor::ExecutorDescriptor>;

        let descriptor = value.descriptor
            .ok_or(ErrorBuilder::new("Executor not set not set"))?;

        let result = match descriptor {
            executor_descriptor::Descriptor::Executable(_) => {
                crate::peer::executor::ExecutorDescriptor::Executable
            },
            executor_descriptor::Descriptor::Container(descriptor) => {
                let Container {
                    engine,
                    name,
                    image,
                    volumes,
                    devices,
                    envs,
                    ports,
                    command,
                    args
                } = descriptor;
                let engine = engine
                    .ok_or(ErrorBuilder::new("Engine not set"))?
                    .try_into()?;
                let name = name
                    .ok_or(ErrorBuilder::new("Container Name not set"))?
                    .try_into()?;
                let image = image
                    .ok_or(ErrorBuilder::new("Container Image not set"))?
                    .try_into()?;
                let volumes = volumes
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;
                let devices = devices
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;
                let envs = envs
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;
                let ports = ports
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;
                let command = command
                    .ok_or(ErrorBuilder::new("Container Command not set"))?
                    .try_into()?;
                let args = args
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;
                crate::peer::executor::ExecutorDescriptor::Container {
                    engine,
                    name,
                    image,
                    volumes,
                    devices,
                    envs,
                    ports,
                    command,
                    args,
                }
            }
        };

        Ok(result)
    }
}

impl From<crate::peer::executor::Engine> for Engine {
    fn from(value: crate::peer::executor::Engine) -> Self {
        match value {
            crate::peer::executor::Engine::Docker => {
                Engine {
                    inner: Some(engine::Inner::Docker(
                        Docker {}
                    ))
                }
            }
            crate::peer::executor::Engine::Podman => {
                Engine {
                    inner: Some(engine::Inner::Podman(
                        Podman {}
                    ))
                }
            }
        }
    }
}

impl TryFrom<Engine> for crate::peer::executor::Engine {
    type Error = ConversionError;

    fn try_from(value: Engine) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<Engine, crate::peer::executor::Engine>;

        let inner = value.inner
            .ok_or(ErrorBuilder::new("Engine not set"))?;

        let result = match inner {
            engine::Inner::Docker(_) => {
                crate::peer::executor::Engine::Docker
            },
            engine::Inner::Podman(_) => {
                crate::peer::executor::Engine::Podman
            },
        };

        Ok(result)
    }
}

impl From<crate::peer::executor::ContainerName> for ContainerName {
    fn from(value: crate::peer::executor::ContainerName) -> Self {
        match value {
            crate::peer::executor::ContainerName::Empty => { Self { value: String::new() } }
            crate::peer::executor::ContainerName::Value(value) => { Self { value } }
        }
    }
}

impl TryFrom<ContainerName> for crate::peer::executor::ContainerName {
    type Error = ConversionError;

    fn try_from(value: ContainerName) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerName, crate::peer::executor::ContainerName>;

        crate::peer::executor::ContainerName::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::executor::ContainerImage> for ContainerImage {
    fn from(value: crate::peer::executor::ContainerImage) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerImage> for crate::peer::executor::ContainerImage {
    type Error = ConversionError;

    fn try_from(value: ContainerImage) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerImage, crate::peer::executor::ContainerImage>;

        crate::peer::executor::ContainerImage::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::executor::ContainerVolume> for ContainerVolume {
    fn from(value: crate::peer::executor::ContainerVolume) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerVolume> for crate::peer::executor::ContainerVolume {
    type Error = ConversionError;

    fn try_from(value: ContainerVolume) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerVolume, crate::peer::executor::ContainerVolume>;

        crate::peer::executor::ContainerVolume::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::executor::ContainerDevice> for ContainerDevice {
    fn from(value: crate::peer::executor::ContainerDevice) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerDevice> for crate::peer::executor::ContainerDevice {
    type Error = ConversionError;

    fn try_from(value: ContainerDevice) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerVolume, crate::peer::executor::ContainerVolume>;

        crate::peer::executor::ContainerDevice::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::executor::ContainerEnvironmentVariable> for ContainerEnvironmentVariable {
    fn from(value: crate::peer::executor::ContainerEnvironmentVariable) -> Self {
        let (name, value) = value.into();
        Self {
            name,
            value
        }
    }
}

impl TryFrom<ContainerEnvironmentVariable> for crate::peer::executor::ContainerEnvironmentVariable {
    type Error = ConversionError;

    fn try_from(value: ContainerEnvironmentVariable) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerEnvironmentVariable, crate::peer::executor::ContainerEnvironmentVariable>;
        
        crate::peer::executor::ContainerEnvironmentVariable::new(value.name, value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::executor::ContainerPortSpec> for ContainerPortSpec {
    fn from(value: crate::peer::executor::ContainerPortSpec) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerPortSpec> for crate::peer::executor::ContainerPortSpec {
    type Error = ConversionError;

    fn try_from(value: ContainerPortSpec) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerPortSpec, crate::peer::executor::ContainerPortSpec>;

        crate::peer::executor::ContainerPortSpec::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl TryFrom<ContainerCommand> for crate::peer::executor::ContainerCommand {
    type Error = ConversionError;

    fn try_from(value: ContainerCommand) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerCommand, crate::peer::executor::ContainerCommand>;

        crate::peer::executor::ContainerCommand::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::executor::ContainerCommand> for ContainerCommand {
    fn from(value: crate::peer::executor::ContainerCommand) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl From<crate::peer::executor::ContainerCommandArgument> for ContainerCommandArgument {
    fn from(value: crate::peer::executor::ContainerCommandArgument) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerCommandArgument> for crate::peer::executor::ContainerCommandArgument {
    type Error = ConversionError;

    fn try_from(value: ContainerCommandArgument) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerCommandArgument, crate::peer::executor::ContainerCommandArgument>;

        crate::peer::executor::ContainerCommandArgument::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}