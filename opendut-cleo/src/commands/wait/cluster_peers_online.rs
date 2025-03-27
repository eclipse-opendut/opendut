use opendut_carl_api::carl::CarlClient;
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
        // TODO: get peer ids of a cluster
        await_peers_online(carl, vec![]).await
    }
}