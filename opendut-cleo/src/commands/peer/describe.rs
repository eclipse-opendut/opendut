use indoc::indoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::{PeerDescriptor, PeerId};
use crate::commands::peer::PeerStatus;
use crate::DescribeOutputFormat;

/// Describe a peer
#[derive(clap::Parser)]
pub struct DescribePeerCli {
    /// ID of the peer
    #[arg()]
    id: PeerId,
}

impl DescribePeerCli {
    pub async fn execute(self, carl: &mut CarlClient, output: DescribeOutputFormat) -> crate::Result<()> {
        let peer_id = self.id;
        
        let peer_descriptor =
            carl.peers.get_peer_descriptor(peer_id).await.map_err(|_| {
                format!("Failed to retrieve peer descriptor for peer <{peer_id}>")
            })?;

        render_peer_descriptor(peer_descriptor, carl, output).await?;
        Ok(())
    }
}

pub async fn render_peer_descriptor(peer_descriptor: PeerDescriptor, carl: &mut CarlClient, output: DescribeOutputFormat) -> crate::Result<()> {
    let peer_devices = peer_descriptor
        .topology
        .devices
        .iter()
        .map(|device| device.name.value())
        .collect::<Vec<_>>()
        .join(", ");
    let text = match output {
        DescribeOutputFormat::Text => {
            let peer_id = peer_descriptor.id;
            let peer_state = carl.peers.get_peer_state(peer_id).await
                .map_err(|error| format!("Failed to retrieve peer state for peer <{peer_id}>.\n  {error}"))?;
            let status = PeerStatus::from(peer_state);

            let network_interfaces = peer_descriptor.network.interfaces.iter()
                .map(|interface| interface.name.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            format!(
                indoc!(
                    "
                Peer: {}
                  Id: {}
                  Location: {}
                  Status: {}
                  NetworkInterfaces: [{}]
                  Devices: [{}]\
            "
                ),
                peer_descriptor.name,
                peer_descriptor.id,
                peer_descriptor.location.unwrap_or_default(),
                status.to_string(),
                network_interfaces,
                peer_devices
            )
        }
        DescribeOutputFormat::Json => serde_json::to_string(&peer_descriptor).unwrap(),
        DescribeOutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&peer_descriptor).unwrap()
        }
    };
    println!("{text}");
    Ok(())
}
