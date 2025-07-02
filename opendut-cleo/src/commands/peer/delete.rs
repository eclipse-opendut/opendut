use std::collections::HashSet;
use std::ops::Not;

use opendut_carl_api::carl::peer::GetPeerDescriptorError;
use opendut_carl_api::carl::{CarlClient, ClientError};
use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;

/// Delete a peer
#[derive(clap::Parser)]
pub struct DeletePeerCli {
    /// ID of the peer
    #[arg()]
    id: PeerId,
    /// Report an error status code when the resource to delete does not exist
    #[arg(long)]
    error_when_missing: bool,
}

impl DeletePeerCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = self.id;

        let peer_descriptor = match carl.peers.get_peer_descriptor(id).await {
            Ok(peer_descriptor) => Ok(peer_descriptor),
            Err(error) => match error {
                ClientError::UsageError(GetPeerDescriptorError::PeerNotFound { .. }) => {
                    eprintln!("No peer descriptor found with ID <{id}>.");
                    if self.error_when_missing {
                        Err(error)
                    } else {
                        return Ok(()); //abort deletion
                    }
                }
                other => Err(other)
            }
            .map_err(|error| format!("Failed to get peer with ID <{id}>.\n  {error}"))
        }?;

        { //block deleting, if device is used in cluster
            let peer_device_ids = peer_descriptor.topology.devices.into_iter()
                .map(|descriptor| descriptor.id)
                .collect::<Vec<_>>();

            let clusters = carl.cluster
                .list_cluster_descriptors()
                .await
                .map_err(|error| format!("Failed to list cluster descriptors.\n  {error}"))?;

            let mut clusters_with_configured_devices: HashSet<ClusterId> = HashSet::new();
            for cluster in clusters {
                for device in cluster.devices {
                    if peer_device_ids.contains(&device) {
                        clusters_with_configured_devices.insert(cluster.id);
                    }
                }
            }
            if clusters_with_configured_devices.is_empty().not() {
                let cluster_list = clusters_with_configured_devices.iter().map(|id| id.to_string()).collect::<Vec<String>>().join(", ");
                Err(format!("Cannot delete peer because it is used in following clusters: {cluster_list}."))?
            }
        }
        
        carl.peers.delete_peer_descriptor(id).await
            .map_err(|error| format!("Failed to delete peer with the id '{id}'.\n  {error}"))?;

        println!("Deleted peer with the PeerID: {id}");

        Ok(())
    }
}
