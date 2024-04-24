use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;

pub async fn execute(carl: &mut CarlClient, id: Uuid) -> crate::Result<()> {
    let id = PeerId::from(id);
    carl.peers
        .delete_peer_descriptor(id)
        .await
        .map_err(|error| format!("Failed to delete peer with the id '{}'.\n  {}", id, error))?;
    println!("Deleted peer with the PeerID: {}", id);

    Ok(())
}
