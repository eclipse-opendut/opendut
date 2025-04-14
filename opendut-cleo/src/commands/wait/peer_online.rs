use std::collections::HashSet;
use std::time::Duration;
use crate::commands::wait::await_peers_online;
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;

/// Wait for a peer to come online
#[derive(clap::Parser)]
pub struct WaitPeerOnlineCli {
    /// IDs of the peers
    #[arg()]
    pub ids: Vec<PeerId>,
    /// Maximum requested observation duration
    #[arg(long, default_value_t = 600)]
    pub max_observation_duration: u64,
    /// Allow to specify peer IDs that may not exist yet
    #[arg(long, default_value_t = false)]
    pub peers_may_not_exist: bool,
}

impl WaitPeerOnlineCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let max_observation_duration = Duration::from_secs(self.max_observation_duration);

        if self.ids.is_empty() {
            Err("No peer IDs provided.".to_string())
        } else {
            await_peers_online(carl, self.ids.into_iter().collect::<HashSet<_>>(), max_observation_duration, self.peers_may_not_exist).await
        }
    }

}