use indoc::formatdoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::{PeerDescriptor, PeerId};
use crate::commands::peer::PeerStatus;
use crate::OutputFormat;

/// Describe a peer
#[derive(clap::Parser)]
pub struct DescribePeerCli {
    /// ID of the peer
    #[arg()]
    id: PeerId,
}

impl DescribePeerCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let peer_id = self.id;
        
        let peer_descriptor = carl.peers.get_peer_descriptor(peer_id)
            .await
            .map_err(|_| {
                format!("Failed to retrieve peer descriptor for peer <{peer_id}>")
            })?;

        render_peer_descriptor(peer_descriptor, carl, output).await
    }
}

pub async fn render_peer_descriptor(peer_descriptor: PeerDescriptor, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
    let text = match output {
        OutputFormat::Table => {
            let peer_id = peer_descriptor.id;
            let name = peer_descriptor.name.value();
            let location = peer_descriptor.location.unwrap_or_default();

            let peer_state = carl.peers.get_peer_state(peer_id).await
                .map_err(|error| format!("Failed to retrieve peer state for peer <{peer_id}>.\n  {error}"))?;
            let status = PeerStatus::from(peer_state);

            let network_interfaces = peer_descriptor.network.interfaces.iter()
                .map(|interface| interface.name.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            let peer_devices = peer_descriptor.topology.devices
                .iter()
                .map(|device| device.name.value())
                .collect::<Vec<_>>()
                .join(", ");

            formatdoc!("
                Peer: {name}
                  Id: {peer_id}
                  Location: {location}
                  Status: {status}
                  NetworkInterfaces: [{network_interfaces}]
                  Devices: [{peer_devices}]\
            ")
        }
        OutputFormat::Json => {
            serde_json::to_string(&peer_descriptor)
                .map_err(|error| format!("Could not serialize peer descriptor.\n  {error}"))?
        },
        OutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&peer_descriptor)
                .map_err(|error| format!("Could not serialize peer descriptor.\n  {error}"))?
        }
    };

    println!("{text}");
    Ok(())
}
