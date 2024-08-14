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
        let id = Some(value.id.into());

        let executor_kind = match value.kind {
            crate::peer::executor::ExecutorKind::Executable => {
                Some(executor_descriptor::Kind::Executable(
                       Executable {}
                    )
                )
            }
            crate::peer::executor::ExecutorKind::Container { 
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
                Some(executor_descriptor::Kind::Container(
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
        };

        ExecutorDescriptor {
            id,
            kind: executor_kind,
            results_url: value.results_url.map(|results_url| results_url.into()),
        }
    }
}

impl TryFrom<ExecutorDescriptor> for crate::peer::executor::ExecutorDescriptor {
    type Error = ConversionError;

    fn try_from(value: ExecutorDescriptor) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ExecutorDescriptor, crate::peer::executor::ExecutorDescriptor>;

        let id = value.id
            .ok_or(ErrorBuilder::field_not_set("id"))?
            .try_into()?;

        let kind = value.kind
            .ok_or(ErrorBuilder::field_not_set("kind"))?;

        let result_kind = match kind {
            executor_descriptor::Kind::Executable(_) => {
                crate::peer::executor::ExecutorKind::Executable
            },
            executor_descriptor::Kind::Container(descriptor) => {
                let Container {
                    engine,
                    name,
                    image,
                    volumes,
                    devices,
                    envs,
                    ports,
                    command,
                    args,
                } = descriptor;
                let engine = engine
                    .ok_or(ErrorBuilder::field_not_set("engine"))?
                    .try_into()?;
                let name = name
                    .ok_or(ErrorBuilder::field_not_set("name"))?
                    .try_into()?;
                let image = image
                    .ok_or(ErrorBuilder::field_not_set("image"))?
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
                    .ok_or(ErrorBuilder::field_not_set("command"))?
                    .try_into()?;
                let args = args
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;
                
                crate::peer::executor::ExecutorKind::Container {
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

        let results_url = value.results_url.map(TryFrom::try_from).transpose()?;

        Ok(
            crate::peer::executor::ExecutorDescriptor {
                id,
                kind: result_kind,
                results_url,
            }
        )
    }
}

impl From<crate::peer::executor::ExecutorId> for ExecutorId {
    fn from(value: crate::peer::executor::ExecutorId) -> Self {
        Self {
            uuid: Some(value.uuid.into())
        }
    }
}

impl TryFrom<ExecutorId> for crate::peer::executor::ExecutorId {
    type Error = ConversionError;

    fn try_from(value: ExecutorId) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ExecutorId, crate::peer::executor::ExecutorId>;

        value.uuid
            .ok_or(ErrorBuilder::field_not_set("uuid"))
            .map(|uuid| Self { uuid: uuid.into() })
    }
}

impl From<crate::peer::executor::container::Engine> for Engine {
    fn from(value: crate::peer::executor::container::Engine) -> Self {
        match value {
            crate::peer::executor::container::Engine::Docker => {
                Engine {
                    inner: Some(engine::Inner::Docker(
                        Docker {}
                    ))
                }
            }
            crate::peer::executor::container::Engine::Podman => {
                Engine {
                    inner: Some(engine::Inner::Podman(
                        Podman {}
                    ))
                }
            }
        }
    }
}

impl TryFrom<Engine> for crate::peer::executor::container::Engine {
    type Error = ConversionError;

    fn try_from(value: Engine) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<Engine, crate::peer::executor::container::Engine>;

        let inner = value.inner
            .ok_or(ErrorBuilder::field_not_set("inner"))?;

        let result = match inner {
            engine::Inner::Docker(_) => {
                crate::peer::executor::container::Engine::Docker
            },
            engine::Inner::Podman(_) => {
                crate::peer::executor::container::Engine::Podman
            },
        };

        Ok(result)
    }
}

impl From<crate::peer::executor::container::ContainerName> for ContainerName {
    fn from(value: crate::peer::executor::container::ContainerName) -> Self {
        match value {
            crate::peer::executor::container::ContainerName::Empty => { Self { value: String::new() } }
            crate::peer::executor::container::ContainerName::Value(value) => { Self { value } }
        }
    }
}

impl TryFrom<ContainerName> for crate::peer::executor::container::ContainerName {
    type Error = ConversionError;

    fn try_from(value: ContainerName) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerName, crate::peer::executor::container::ContainerName>;

        crate::peer::executor::container::ContainerName::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<crate::peer::executor::container::ContainerImage> for ContainerImage {
    fn from(value: crate::peer::executor::container::ContainerImage) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerImage> for crate::peer::executor::container::ContainerImage {
    type Error = ConversionError;

    fn try_from(value: ContainerImage) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerImage, crate::peer::executor::container::ContainerImage>;

        crate::peer::executor::container::ContainerImage::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<crate::peer::executor::container::ContainerVolume> for ContainerVolume {
    fn from(value: crate::peer::executor::container::ContainerVolume) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerVolume> for crate::peer::executor::container::ContainerVolume {
    type Error = ConversionError;

    fn try_from(value: ContainerVolume) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerVolume, crate::peer::executor::container::ContainerVolume>;

        crate::peer::executor::container::ContainerVolume::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<crate::peer::executor::container::ContainerDevice> for ContainerDevice {
    fn from(value: crate::peer::executor::container::ContainerDevice) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerDevice> for crate::peer::executor::container::ContainerDevice {
    type Error = ConversionError;

    fn try_from(value: ContainerDevice) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerVolume, crate::peer::executor::container::ContainerVolume>;

        crate::peer::executor::container::ContainerDevice::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<crate::peer::executor::container::ContainerEnvironmentVariable> for ContainerEnvironmentVariable {
    fn from(value: crate::peer::executor::container::ContainerEnvironmentVariable) -> Self {
        let (name, value) = value.into();
        Self {
            name,
            value
        }
    }
}

impl TryFrom<ContainerEnvironmentVariable> for crate::peer::executor::container::ContainerEnvironmentVariable {
    type Error = ConversionError;

    fn try_from(value: ContainerEnvironmentVariable) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerEnvironmentVariable, crate::peer::executor::container::ContainerEnvironmentVariable>;
        
        crate::peer::executor::container::ContainerEnvironmentVariable::new(value.name, value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<crate::peer::executor::container::ContainerPortSpec> for ContainerPortSpec {
    fn from(value: crate::peer::executor::container::ContainerPortSpec) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerPortSpec> for crate::peer::executor::container::ContainerPortSpec {
    type Error = ConversionError;

    fn try_from(value: ContainerPortSpec) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerPortSpec, crate::peer::executor::container::ContainerPortSpec>;

        crate::peer::executor::container::ContainerPortSpec::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl TryFrom<ContainerCommand> for crate::peer::executor::container::ContainerCommand {
    type Error = ConversionError;

    fn try_from(value: ContainerCommand) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerCommand, crate::peer::executor::container::ContainerCommand>;

        crate::peer::executor::container::ContainerCommand::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<crate::peer::executor::container::ContainerCommand> for ContainerCommand {
    fn from(value: crate::peer::executor::container::ContainerCommand) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl From<crate::peer::executor::container::ContainerCommandArgument> for ContainerCommandArgument {
    fn from(value: crate::peer::executor::container::ContainerCommandArgument) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ContainerCommandArgument> for crate::peer::executor::container::ContainerCommandArgument {
    type Error = ConversionError;

    fn try_from(value: ContainerCommandArgument) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ContainerCommandArgument, crate::peer::executor::container::ContainerCommandArgument>;

        crate::peer::executor::container::ContainerCommandArgument::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

impl From<crate::peer::executor::ResultsUrl> for ResultsUrl {
    fn from(value: crate::peer::executor::ResultsUrl) -> Self {
        Self {
            value: value.into()
        }
    }
}

impl TryFrom<ResultsUrl> for crate::peer::executor::ResultsUrl{
    type Error = ConversionError;

    fn try_from(value: ResultsUrl) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ResultsUrl, crate::peer::executor::ResultsUrl>;

        crate::peer::executor::ResultsUrl::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}
