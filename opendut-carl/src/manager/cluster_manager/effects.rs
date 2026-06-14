use tracing::{error, info};
use opendut_model::peer::state::PeerConnectionState;
use crate::manager::cluster_manager::ClusterManagerRef;
use crate::resource::manager::{ResourceManagerRef, SubscriptionEvent};

pub(crate) async fn register(resource_manager: ResourceManagerRef, cluster_manager: ClusterManagerRef) {
    schedule_redeploying_clusters_when_all_peers_become_available(resource_manager.clone(), cluster_manager.clone()).await;
}

async fn schedule_redeploying_clusters_when_all_peers_become_available(resource_manager: ResourceManagerRef, self_ref: ClusterManagerRef) {
    resource_manager.spawn_event_listener::<PeerConnectionState>(async move |event| {
        if let SubscriptionEvent::Inserted { id: peer_id, value: PeerConnectionState::Online { remote_host } } = event {
            info!("Peer <{peer_id}> is now online with remote address <{remote_host}>. Checking if any clusters can now be deployed...");

            let mut self_ref = self_ref.lock().await;
            let result = self_ref.rollout_all_clusters_containing_newly_available_peer(peer_id).await;
            if let Err(error) = result {
                error!("Error while attempting deployment of clusters in which newly available peer <{peer_id}> is contained:  \n{error}");
            }
        }
    }).await;
}
