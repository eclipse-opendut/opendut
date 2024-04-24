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
