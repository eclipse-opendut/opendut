use indoc::indoc;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::{PeerDescriptor, PeerId};
use crate::DescribeOutputFormat;

/// Describe a peer
#[derive(clap::Parser)]
pub struct DescribePeerCli {
    ///PeerID
    #[arg()]
    id: Uuid,
}

impl DescribePeerCli {
    pub async fn execute(self, carl: &mut CarlClient, output: DescribeOutputFormat) -> crate::Result<()> {
        let peer_id = PeerId::from(self.id);
        
        let peer_descriptor =
            carl.peers.get_peer_descriptor(peer_id).await.map_err(|_| {
                format!("Failed to retrieve peer descriptor for peer <{}>", peer_id)
            })?;

        render_peer_descriptor(peer_descriptor, output, );
        Ok(())
    }
}

pub fn render_peer_descriptor(peer_descriptor: PeerDescriptor, output: DescribeOutputFormat) {
    let peer_devices = peer_descriptor
        .topology
        .devices
        .iter()
        .map(|device| device.name.value())
        .collect::<Vec<_>>()
        .join(", ");
    let text = match output {
        DescribeOutputFormat::Text => {
            format!(
                indoc!(
                    "
                Peer: {}
                  Id: {}
                  Devices: [{}]\
            "
                ),
                peer_descriptor.name, peer_descriptor.id, peer_devices
            )
        }
        DescribeOutputFormat::Json => serde_json::to_string(&peer_descriptor).unwrap(),
        DescribeOutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&peer_descriptor).unwrap()
        }
    };
    println!("{text}");
}
