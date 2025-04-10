use tracing::log::{error, trace};
use opendut_types::peer::PeerDescriptor;
use crate::manager::peer_messaging_broker::PeerMessagingBrokerRef;
use crate::resource::manager::{ResourceManagerRef, SubscriptionEvent};

pub(crate) async fn register(resource_manager: ResourceManagerRef, peer_messaging_broker: PeerMessagingBrokerRef) {
    disconnect_peer_when_removed(resource_manager.clone(), peer_messaging_broker.clone()).await;
}

pub async fn disconnect_peer_when_removed(resource_manager: ResourceManagerRef, peer_messaging_broker: PeerMessagingBrokerRef) {
    let mut peer_subscription = resource_manager.subscribe::<PeerDescriptor>().await;

    tokio::spawn(async move {
        loop {
            let peer_subscription = peer_subscription.receive().await;

            if let Ok(SubscriptionEvent::Removed { id: peer_id, ..  }) = peer_subscription {
                trace!("Peer <{peer_id}> was removed. Checking if there is an open peer connection.");
                let _ = peer_messaging_broker.remove_peer(peer_id).await
                    .inspect_err(|error| error!("Failed to remove peer <{peer_id}>: {error}"));
            }
        }
    });
}
