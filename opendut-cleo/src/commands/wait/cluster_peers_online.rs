use opendut_carl_api::carl::CarlClient;
use opendut_carl_api::carl::cluster::{ListClusterPeerStatesResponse};
use opendut_types::cluster::ClusterId;
use crate::commands::wait::await_peers_online;

/// Wait for all peers in a cluster to come online
#[derive(clap::Parser)]
pub struct WaitPeersInClusterOnline {
    /// ID of the cluster
    #[arg()]
    pub id: ClusterId,
}

impl WaitPeersInClusterOnline {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let response = carl.cluster.list_cluster_peer_states(self.id).await
            .map_err(|cause| cause.to_string())?;

        match response {
            ListClusterPeerStatesResponse::Success { peer_states } => {
                let peer_ids = peer_states.keys().cloned().collect::<Vec<_>>();
                await_peers_online(carl, peer_ids).await
            }
            ListClusterPeerStatesResponse::Failure { message } => {
                Err(format!("Failed to list peer states: {message}"))
            }
        }
    }
}