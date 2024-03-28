pub mod list {
    use cli_table::{print_stdout, Table, WithTitle};
    use serde::Serialize;
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::{PeerDescriptor, PeerId};
    use opendut_types::peer::executor::{ContainerImage, Engine, ExecutorDescriptor};

    use crate::{ListOutputFormat};

    #[derive(Table, Debug, Serialize)]
    struct ContainerExecutorTable {
        #[table(title = "Engine")]
        engine: Engine,
        #[table(title = "Name")]
        name: String,
        #[table(title = "Image")]
        image: ContainerImage,
        #[table(title = "Volume")]
        volumes: String,
        #[table(title = "Devices")]
        devices: String,
        #[table(title = "Envs")]
        envs: String,
        #[table(title = "Ports")]
        ports: String,
        #[table(title = "Command")]
        command: String,
        #[table(title = "Args")]
        args: String,
    }

    pub async fn execute(carl: &mut CarlClient, id: Uuid, output: ListOutputFormat) -> crate::Result<()> {
        let peer_id = PeerId::from(id);
        let peer = carl
            .peers.get_peer_descriptor(peer_id)
            .await
            .map_err(|error| format!("Could not find peer.\n  {}", error))?;
        let executor_table = filter_connected_peers(&peer);

        match output {
            ListOutputFormat::Table => {
                print_stdout(executor_table.with_title())
                    .expect("List of executors should be printable as table.");
            }
            ListOutputFormat::Json => {
                let json = serde_json::to_string(&executor_table).unwrap();
                println!("{}", json);
            }
            ListOutputFormat::PrettyJson => {
                let json = serde_json::to_string_pretty(&executor_table).unwrap();
                println!("{}", json);
            }
        }
        Ok(())
    }

    fn filter_connected_peers(
        peer: &PeerDescriptor,
    ) -> Vec<ContainerExecutorTable> {
        let mut executor_table = vec![];
        for executor in &peer.executors.executors {
            if let ExecutorDescriptor::Container {
                engine,
                name,
                image,
                volumes,
                devices,
                envs,
                ports,
                command,
                args
            } = executor {
                let args = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
                let volumes = volumes.iter().map(|volume| volume.to_string()).collect::<Vec<_>>();
                let devices = devices.iter().map(|device| device.to_string()).collect::<Vec<_>>();
                let ports = ports.iter().map(|port| port.to_string()).collect::<Vec<_>>();
                let envs = envs.iter().map(|env|
                    format!("{}={}", env.name(), env.value())).collect::<Vec<_>>();
                executor_table.push(ContainerExecutorTable {
                    engine: Clone::clone(engine),
                    name: name.into(),
                    image: Clone::clone(image),
                    volumes: volumes.join(", "),
                    devices: devices.join(", "),
                    envs: envs.join(", "),
                    ports: ports.join(", "),
                    command: command.into(),
                    args: args.join(", "),
                });
            }
        };
        executor_table
    }
}

pub mod create {
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::{PeerId};
    use opendut_types::peer::executor::{ContainerCommand, ContainerCommandArgument, ContainerDevice, ContainerEnvironmentVariable, ContainerImage, ContainerName, ContainerPortSpec, ContainerVolume, Engine, ExecutorDescriptor};

    use crate::{CreateOutputFormat, DescribeOutputFormat, EngineVariants};
    
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        carl: &mut CarlClient,
        peer_id: Uuid,
        engine: EngineVariants,
        name: Option<ContainerName>,
        image: ContainerImage,
        volumes:  Option<Vec<ContainerVolume>>,
        devices: Option<Vec<ContainerDevice>>,
        envs: Option<Vec<String>>,
        ports: Option<Vec<ContainerPortSpec>>,
        command: Option<ContainerCommand>,
        args: Option<Vec<ContainerCommandArgument>>,
        output: CreateOutputFormat,
    ) -> crate::Result<()> {
        let peer_id = PeerId::from(peer_id);

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;
        
        let engine = match engine {
            EngineVariants::Docker => { Engine::Docker }
            EngineVariants::Podman => { Engine::Podman }
        };
        
        let volumes = volumes.unwrap_or_default();
        let devices = devices.unwrap_or_default();
        let ports = ports.unwrap_or_default();
        let args = args.unwrap_or_default();
        
        let mut environment_variables = vec![];
        
        for env in envs.unwrap_or_default() {
            if let Some((name, value)) = env.split_once('=') {
                let env = ContainerEnvironmentVariable::new(name, value)
                    .map_err(|cause| cause.to_string())?;
                environment_variables.push(env)
            }
        };
        
        let executor_descriptor = ExecutorDescriptor::Container {
            engine,
            name: name.unwrap_or_default(),
            image,
            volumes,
            devices,
            envs: environment_variables,
            ports,
            command: command.unwrap_or_default(),
            args,
        };

        peer_descriptor.executors.executors.push(executor_descriptor);
        
        carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
            .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
        let output_format = DescribeOutputFormat::from(output);
        crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, output_format);

        Ok(())
    }
}

pub mod delete {
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::executor::{ContainerImage, ExecutorDescriptor};
    use opendut_types::peer::PeerId;

    pub async fn execute(carl: &mut CarlClient, id: Uuid, images: Vec<ContainerImage>) -> crate::Result<()> {
        let id = PeerId::from(id);

        let mut peer = carl.peers
            .get_peer_descriptor(id)
            .await
            .map_err(|error| format!("Failed to get peer with the id '{}'.\n  {}", id, error))?;

        let container_images = images.into_iter()
            .map(ContainerImage::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        for container_image in container_images {
            peer.executors.executors.retain(|executor| match executor {
                ExecutorDescriptor::Executable => true,
                ExecutorDescriptor::Container { image, .. } => {
                    image != &container_image
                }
            } )
        };

        carl.peers.store_peer_descriptor(peer).await
            .map_err(|error| format!("Failed to delete container executor for peer.\n  {}", error))?;

        Ok(())
    }
}
