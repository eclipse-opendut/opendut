use crate::commands::wait::await_peers_online;
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;

/// Wait for a peer to come online
#[derive(clap::Parser)]
pub struct WaitPeerOnlineCli {
    /// IDs of the peers
    #[arg()]
    pub ids: Vec<PeerId>,
}

impl WaitPeerOnlineCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        if self.ids.is_empty() {
            Err("No peer IDs provided.".to_string())
        } else {
            await_peers_online(carl, self.ids).await
        }
    }

}