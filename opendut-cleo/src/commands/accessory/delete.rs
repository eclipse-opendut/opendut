use uuid::Uuid;
use opendut_carl_api::carl::CarlClient;
use opendut_types::topology::AccessoryId;

/// Delete a device
#[derive(clap::Parser)]
pub struct DeleteAccessoryCli {
    ///AccessoryId
    #[arg()]
    id: Uuid,
}

impl DeleteAccessoryCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let accessory_id = AccessoryId::from(self.id);

        let mut peers = carl.peers.list_peer_descriptors().await
            .map_err(|error| format!("Could not list peers.\n  {}", error))?;

        let peer = peers.iter_mut().find(|peer| {
            peer.topology.accessories
                .iter()
                .any(|accessory| accessory.id == accessory_id)
        }).ok_or(format!("Cannot find a peer with the accessory <{}>.", accessory_id))?;

        peer.topology.accessories.retain(|accessory| accessory.id != accessory_id);

        carl.peers.store_peer_descriptor(Clone::clone(peer)).await
            .map_err(|error| format!("Failed to delete peer.\n  {}", error))?;

        Ok(())
    }
}
