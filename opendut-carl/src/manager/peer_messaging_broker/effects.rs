use tracing::log::trace;
use opendut_types::peer::PeerDescriptor;
use opendut_types::peer::state::PeerConnectionState;
use crate::manager::peer_messaging_broker::{PeerMessagingBrokerRef};
use crate::resource::manager::{ResourceManagerRef, SubscriptionEvent};

pub(crate) async fn register(resource_manager: ResourceManagerRef, peer_messaging_broker: PeerMessagingBrokerRef) {
    disconnect_peer_when_removed(resource_manager.clone(), peer_messaging_broker.clone()).await;
}

pub async fn disconnect_peer_when_removed(resource_manager: ResourceManagerRef, _peer_messaging_broker: PeerMessagingBrokerRef) {
    let mut peer_subscription = resource_manager.subscribe::<PeerDescriptor>().await;

    tokio::spawn(async move {
        loop {
            let peer_subscription = peer_subscription.receive().await;

            if let Ok(SubscriptionEvent::Removed { id: peer_id, ..  }) = peer_subscription {
                trace!("Peer <{peer_id}> was removed. Running trigger task...");
                let peer_connection_state = resource_manager.remove::<PeerConnectionState>(peer_id).await;
                if let Ok(Some(peer_connection_state)) = peer_connection_state {
                    match peer_connection_state {
                        PeerConnectionState::Offline => {}
                        PeerConnectionState::Online { .. } => {
                            // TODO: disconnect peer
                        }
                    }
                    trace!("Obsolete connection state for peer <{peer_id}> was removed.");
                }

            }
        }
    });
}
