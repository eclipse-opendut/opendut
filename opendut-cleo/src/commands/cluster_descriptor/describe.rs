use indoc::indoc;

use opendut_carl_api::carl::CarlClient;
use opendut_types::cluster::{ClusterId, ClusterName};
use opendut_types::peer::{PeerId, PeerName};
use opendut_types::topology::DeviceName;
use serde::Serialize;

use crate::DescribeOutputFormat;


/// Describe a cluster descriptor
#[derive(clap::Parser)]
pub struct DescribeClusterDescriptorCli {
    /// ID of the cluster
    #[arg()]
    id: ClusterId,
}

#[derive(Debug, Serialize)]
struct ClusterTable {
    name: ClusterName,
    id: ClusterId,
    leader: PeerId,
    peers: Vec<PeerName>,
    devices: Vec<DeviceName>,
}

impl DescribeClusterDescriptorCli {
    pub async fn execute(self, carl: &mut CarlClient, output: DescribeOutputFormat) -> crate::Result<()> {
        let cluster_id = self.id;

        let clusters_configuration = carl.cluster.list_cluster_descriptors().await
            .map_err(|_| String::from("Failed to get list of cluster descriptors!"))?;

        let cluster_descriptor = clusters_configuration.into_iter()
            .find(|cluster_descriptor| cluster_descriptor.id == cluster_id)
            .ok_or(format!("Failed to find cluster descriptor for ClusterID <{cluster_id}>"))?;

        let cluster_devices = {
            let devices = carl.peers.list_devices().await
                .map_err(|_| String::from("Failed to get list of devices!"))?;
            devices.into_iter()
                .filter(|device| cluster_descriptor.devices.contains(&device.id))
                .map(|devices| devices.name)
                .collect::<Vec<_>>()
        };

        let cluster_peers = {
            let peers = carl.peers.list_peer_descriptors().await
                .map_err(|_| String::from("Failed to get list of peers!"))?;
            peers.into_iter()
                .filter(|peer| {
                    peer.topology.devices.iter().any(|device| cluster_devices.contains(&device.name))
                })
                .map(|peer| peer.name)
                .collect::<Vec<_>>()
        };

        let table = ClusterTable {
            name: cluster_descriptor.name,
            id: cluster_id,
            leader: cluster_descriptor.leader,
            peers: cluster_peers,
            devices: cluster_devices,
        };

        let text = match output {
            DescribeOutputFormat::Text => {
                format!(indoc!("
                Cluster Descriptor: {}
                  Id: {}
                  Leader: {}
                  Peers: [{:?}]
                  Devices: [{:?}]
            "), table.name, table.id, table.leader, table.peers, table.devices)
            }
            DescribeOutputFormat::Json => {
                serde_json::to_string(&table).unwrap()
            }
            DescribeOutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&table).unwrap()
            }
        };

        println!("{text}");

        Ok(())
    }
}
