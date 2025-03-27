use crate::commands::wait::await_peers_online;
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;

/// Wait for a peer to come online
#[derive(clap::Parser)]
pub struct WaitPeerOnlineCli {
    /// ID of the peer
    #[arg()]
    pub id: PeerId,
}

impl WaitPeerOnlineCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        await_peers_online(carl, vec![self.id]).await
    }

}