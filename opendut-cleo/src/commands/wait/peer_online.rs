use std::collections::HashSet;
use std::time::Duration;
use crate::commands::wait::await_peers_online;
use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::PeerId;

/// Wait for a peer to come online
#[derive(clap::Parser)]
pub struct WaitPeerOnlineCli {
    /// IDs of the peers
    #[arg()]
    pub ids: Vec<PeerId>,
    /// Maximum observation duration in seconds
    #[arg(long, default_value_t = 600)]
    pub timeout: u64,
    /// Allow to specify peer IDs that haven't been created yet
    #[arg(long, default_value_t = false)]
    pub peers_may_not_yet_exist: bool,
}

impl WaitPeerOnlineCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let max_observation_duration = Duration::from_secs(self.timeout);

        if self.ids.is_empty() {
            Err("No peer IDs provided.".to_string())
        } else {
            await_peers_online(carl, self.ids.into_iter().collect::<HashSet<_>>(), max_observation_duration, self.peers_may_not_yet_exist).await
        }
    }

}
