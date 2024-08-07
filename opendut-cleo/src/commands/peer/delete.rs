use std::ops::Not;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;

/// Delete a peer
#[derive(clap::Parser)]
pub struct DeletePeerCli {
    ///PeerID
    #[arg()]
    id: Uuid,
}

impl DeletePeerCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = PeerId::from(self.id);
        
        let peer_descriptor = carl.peers.get_peer_descriptor(id).await
            .map_err(|error| format!("Failed to get peer descriptor for peer: {}.\n {}", id, error))?;
        
        let peer_device_ids = peer_descriptor.topology.devices.into_iter().map(|descriptor| descriptor.id.0).collect::<Vec<_>>();
        
        let clusters = carl.cluster
            .list_cluster_configurations()
            .await
            .map_err(|error| format!("Failed to list cluster configurations.\n  {}", error))?;
        
        let mut clusters_with_configured_devices: Vec<String> = vec![];
        for cluster in clusters {
            for device in cluster.devices {
                if peer_device_ids.contains(&device.0) {
                    clusters_with_configured_devices.push(cluster.id.to_string());
                }
            }
        }
        if clusters_with_configured_devices.is_empty().not() {
            Err(format!("Cannot delete peer because it is used in following clusters: {}", clusters_with_configured_devices.join(", ")))?
        }
        
        carl.peers
            .delete_peer_descriptor(id)
            .await
            .map_err(|error| format!("Failed to delete peer with the id '{}'.\n  {}", id, error))?;
        println!("Deleted peer with the PeerID: {}", id);

        Ok(())
    }
}
