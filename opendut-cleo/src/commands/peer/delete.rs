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
        carl.peers
            .delete_peer_descriptor(id)
            .await
            .map_err(|error| format!("Failed to delete peer with the id '{}'.\n  {}", id, error))?;
        println!("Deleted peer with the PeerID: {}", id);

        Ok(())
    }
}