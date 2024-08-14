use cli_table::{print_stdout, Table, WithTitle};
use opendut_types::peer::executor::ExecutorDescriptor;
use serde::Serialize;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::peer::executor::{container::{ContainerImage, Engine}, ExecutorKind};

use crate::{ListOutputFormat};

/// List all container executors for one peer
#[derive(clap::Parser)]
pub struct ListContainerExecutorCli {
    ///PeerID
    #[arg(short, long)]
    pub id: Uuid,
}

impl ListContainerExecutorCli {
    pub async fn execute(self, carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
        let peer_id = PeerId::from(self.id);
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
}

fn filter_connected_peers(
    peer: &PeerDescriptor,
) -> Vec<ContainerExecutorTable> {
    let mut executor_table = vec![];
    for executor in &peer.executors.executors {
        let ExecutorDescriptor { id: _, kind, results_url } = executor;
        
        if let ExecutorKind::Container {
            engine,
            name,
            image,
            volumes,
            devices,
            envs,
            ports,
            command,
            args,
        } = kind {
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
                results_url: results_url.clone().map_or("None".to_string(), |results_url| results_url.into()),
            });
        }
    };
    executor_table
}

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
    #[table(title = "Results URL")]
    results_url: String,
}
