use indoc::formatdoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::cluster::{ClusterDescriptor, ClusterId};

use crate::OutputFormat;


/// Describe a cluster descriptor
#[derive(clap::Parser)]
pub struct DescribeClusterDescriptorCli {
    /// ID of the cluster
    #[arg()]
    id: ClusterId,
}

impl DescribeClusterDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let cluster_id = self.id;

        let cluster_descriptor = carl.cluster.get_cluster_descriptor(cluster_id)
            .await
            .map_err({
                |_| format!("Failed to retrieve cluster descriptor for cluster <{cluster_id}>")
            })?;

        render_cluster_descriptor(cluster_descriptor, carl, output).await
    }
}

async fn render_cluster_descriptor(cluster_descriptor: ClusterDescriptor, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
    let text = match output {
        OutputFormat::Table => {
            let cluster_id = cluster_descriptor.id;
            let name = cluster_descriptor.name.to_string();
            let leader = cluster_descriptor.leader;

            let (device_ids, device_names): (Vec<_>, String)  = {
                let devices = carl.peers.list_devices().await
                    .map_err(|_| String::from("Failed to get list of devices!"))?;

                let devices: (Vec<_>, Vec<_>) = devices.into_iter()
                    .filter(|device| cluster_descriptor.devices.contains(&device.id))
                    .map(|devices| (devices.id, devices.name.to_string()))
                    .unzip();

                (devices.0, devices.1.join(", "))
            };

            let cluster_peers = {
                let peers = carl.peers.list_peer_descriptors().await
                    .map_err(|_| String::from("Failed to get list of peers!"))?;

                let peers = peers.into_iter()
                    .filter(|peer| {
                        peer.topology.devices.iter().any(|peer_device| device_ids.contains(&peer_device.id))
                    })
                    .map(|peer| peer.name.to_string())
                    .collect::<Vec<_>>();

                peers.join(", ")
            };

            formatdoc!("
                Cluster Descriptor: {name}
                  Id: {cluster_id}
                  Leader: {leader}
                  Peers: [{cluster_peers}]
                  Devices: [{device_names}]\
            ")
        }
        OutputFormat::Json => {
            serde_json::to_string(&cluster_descriptor)
                .map_err(|error| format!("Could not serialize cluster descriptor.\n  {error}"))?
        }
        OutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&cluster_descriptor)
                .map_err(|error| format!("Could not serialize cluster descriptor.\n  {error}"))?
        }
    };

    println!("{text}");
    Ok(())
}
