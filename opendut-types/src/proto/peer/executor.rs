use crate::proto::{conversion, ConversionError, ConversionErrorBuilder, ConversionResult};

crate::include_proto!("opendut.types.peer.executor");


conversion! {
    type Model = crate::peer::executor::ExecutorDescriptors;
    type Proto = ExecutorDescriptors;

    fn from(value: Model) -> Proto {
        Proto {
            executors: value
                .executors
                .into_iter()
                .map(ExecutorDescriptor::from)
                .collect(),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        value
            .executors
            .into_iter()
            .map(ExecutorDescriptor::try_into)
            .collect::<Result<_, _>>()
            .map(|executors| Model { executors } )
    }
}


conversion! {
    type Model = crate::peer::executor::ExecutorDescriptor;
    type Proto = ExecutorDescriptor;

    fn from(value: Model) -> Proto {
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

        Proto {
            id,
            kind: executor_kind,
            results_url: value.results_url.map(|results_url| results_url.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {

        let id = extract!(value.id)?.try_into()?;

        let kind = extract!(value.kind)?;

        let result_kind = match kind {
            executor_descriptor::Kind::Executable(_) => {
                crate::peer::executor::ExecutorKind::Executable
            },
            executor_descriptor::Kind::Container(descriptor) => {
                let Container { engine, name, image, volumes, devices, envs, ports, command, args } = descriptor;

                let engine = extract!(engine)?.try_into()?;
                let name = extract!(name)?.try_into()?;
                let image = extract!(image)?.try_into()?;

                let volumes = volumes.into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;

                let devices = devices.into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;

                let envs = envs.into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;

                let ports = ports.into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;

                let command = extract!(command)?.try_into()?;

                let args = args.into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<_, _>>()?;

                crate::peer::executor::ExecutorKind::Container { engine, name, image, volumes, devices, envs, ports, command, args}
            }
        };

        let results_url = value.results_url.map(TryFrom::try_from).transpose()?;

        Ok(Model {
            id,
            kind: result_kind,
            results_url,
        })
    }
}

conversion! {
    type Model = crate::peer::executor::ExecutorId;
    type Proto = ExecutorId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.uuid.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| Model { uuid: uuid.into() })
    }
}

conversion! {
    type Model = crate::peer::executor::container::Engine;
    type Proto = Engine;

    fn from(value: Model) -> Proto {
        match value {
            Model::Docker => {
                Engine {
                    inner: Some(engine::Inner::Docker(
                        Docker {}
                    ))
                }
            }
            Model::Podman => {
                Engine {
                    inner: Some(engine::Inner::Podman(
                        Podman {}
                    ))
                }
            }
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {

        let inner = extract!(value.inner)?;

        let result = match inner {
            engine::Inner::Docker(_) => Model::Docker,
            engine::Inner::Podman(_) => Model::Podman,
        };

        Ok(result)
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerName;
    type Proto = ContainerName;

    fn from(value: Model) -> Proto {
        match value {
            Model::Empty => { Proto { value: String::new() } }
            Model::Value(value) => { Proto { value } }
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerImage;
    type Proto = ContainerImage;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.into()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerVolume;
    type Proto = ContainerVolume;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.into()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerDevice;
    type Proto = ContainerDevice;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.into()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerEnvironmentVariable;
    type Proto = ContainerEnvironmentVariable;

    fn from(value: Model) -> Proto {
        let (name, value) = value.into();
        Proto {
            name,
            value
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::new(value.name, value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerPortSpec;
    type Proto = ContainerPortSpec;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.into()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerCommand;
    type Proto = ContainerCommand;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.into()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::container::ContainerCommandArgument;
    type Proto = ContainerCommandArgument;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.into()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::executor::ResultsUrl;
    type Proto = ResultsUrl;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.into()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}
