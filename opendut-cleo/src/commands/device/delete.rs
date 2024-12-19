use std::ops::Not;
use opendut_carl_api::carl::CarlClient;
use opendut_types::topology::DeviceId;

/// Delete a device
#[derive(clap::Parser)]
pub struct DeleteDeviceCli {
    /// ID of the device
    #[arg()]
    id: DeviceId,
}

impl DeleteDeviceCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let device_to_delete = self.id;

        let mut peers = carl.peers.list_peer_descriptors().await
            .map_err(|error| format!("Could not list peers.\n  {}", error))?;

        let peer = peers.iter_mut().find(|peer| {
            peer.topology.devices
                .iter()
                .any(|device| device.id == device_to_delete)
        }).ok_or(format!("Cannot find a peer with the device <{}>.", device_to_delete))?;

        { //block deleting, if device is used in cluster
            let clusters = carl.cluster
                .list_cluster_configurations()
                .await
                .map_err(|error| format!("Failed to list cluster configurations.\n  {}", error))?;

            let mut devices_in_cluster: Vec<String> = vec![];
            for cluster in clusters {
                for device in cluster.devices {
                    if device == device_to_delete {
                        devices_in_cluster.push(cluster.id.to_string());
                    }
                }
            }
            if devices_in_cluster.is_empty().not() {
                Err(format!("Cannot delete device because it is used in following clusters: {}", devices_in_cluster.join(", ")))?
            }
        }

        peer.topology.devices.retain(|device| device.id != device_to_delete);

        carl.peers.store_peer_descriptor(Clone::clone(peer)).await
            .map_err(|error| format!("Failed to delete peer.\n  {}", error))?;

        Ok(())
    }
}
