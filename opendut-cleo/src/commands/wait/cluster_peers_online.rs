use std::collections::HashSet;
use std::time::Duration;
use opendut_carl_api::carl::CarlClient;
use opendut_carl_api::carl::cluster::{ListClusterPeerStatesResponse};
use opendut_model::cluster::ClusterId;
use crate::commands::wait::await_peers_online;

/// Wait for all peers in a cluster to come online
#[derive(clap::Parser)]
pub struct WaitPeersInClusterOnline {
    /// ID of the cluster
    #[arg()]
    pub id: ClusterId,
    /// Maximum observation duration in seconds
    #[arg(long, default_value_t = 600)]
    pub timeout: u64,
    /// Allow to specify peer IDs that haven't been created yet
    #[arg(long, default_value_t = false)]
    pub peers_may_not_yet_exist: bool,
}

impl WaitPeersInClusterOnline {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let response = carl.cluster.list_cluster_peer_states(self.id).await
            .map_err(|cause| cause.to_string())?;

        let max_observation_duration = Duration::from_secs(self.timeout);
        match response {
            ListClusterPeerStatesResponse::Success { peer_states } => {
                let peer_ids = peer_states.keys().cloned().collect::<HashSet<_>>();
                await_peers_online(carl, peer_ids, max_observation_duration, self.peers_may_not_yet_exist).await
            }
            ListClusterPeerStatesResponse::Failure { message } => {
                Err(format!("Failed to list peer states: {message}"))
            }
        }
    }
}
