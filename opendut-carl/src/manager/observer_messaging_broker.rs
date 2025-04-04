use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, warn, trace};
use opendut_carl_api::carl::observer::WaitForPeersOnlineResponse;
use opendut_types::peer::PeerId;
use opendut_types::peer::state::PeerConnectionState;
use crate::manager::cluster_manager::ClusterManagerRef;
use crate::resource::manager::{ResourceManagerRef, SubscriptionEvent};
use crate::resource::persistence::error::PersistenceError;

pub type ObserverMessagingBrokerRef = Arc<ObserverMessagingBroker>;


pub struct ObserverMessagingBroker {
    resource_manager: ResourceManagerRef,
    #[expect(unused)]
    cluster_manager: ClusterManagerRef,
    // TODO: limit number of observers? maybe use client_id or session_id
}

#[derive(Debug, thiserror::Error)]
pub enum ObserverError {
    #[error("Persistence could not be accessed while determining the connection status")]
    Persistence(#[from] PersistenceError),
}

impl ObserverMessagingBroker {
    pub fn new(resource_manager: ResourceManagerRef, cluster_manager: ClusterManagerRef) -> ObserverMessagingBrokerRef {
        Arc::new(Self {
            resource_manager,
            cluster_manager,

        })
    }
    
    async fn determine_connection_states(&self, peer_ids: &HashSet<PeerId>) -> Result<HashMap<PeerId, Option<PeerConnectionState>>, PersistenceError> {
        let all_peer_connection_states = self.resource_manager.list::<PeerConnectionState>().await?;
        
        let selected_peer_connection_states = peer_ids.iter().map(|peer_id| {
            let peer_connection_state = all_peer_connection_states.get(peer_id);
            match peer_connection_state {
                None => {
                    (*peer_id, None)
                }
                Some(connection_state) => {
                    (*peer_id, Some(connection_state.clone()))
                }
            }
        }).collect::<HashMap<_, _>>();
        
        Ok(selected_peer_connection_states)
    }
    
    pub async fn wait_for_peers_online(&self, peer_ids: HashSet<PeerId>) -> Result<mpsc::Receiver<opendut_carl_api::proto::services::observer_messaging_broker::WaitForPeersOnlineResponse>, ObserverError> {
        let (tx_outbound, rx_outbound) = mpsc::channel::<opendut_carl_api::proto::services::observer_messaging_broker::WaitForPeersOnlineResponse>(1024);
        let peer_connection_states = self.determine_connection_states(&peer_ids).await
            .map_err(ObserverError::Persistence)?;
        let resource_manager = self.resource_manager.clone();

        tokio::spawn(async move {
            let mut peer_state_subscription = resource_manager.subscribe::<PeerConnectionState>().await;

            let mut offline_peers = peer_connection_states.into_iter()
                .filter(|(_, peer_connection_state)| {
                    peer_connection_state.is_none() || matches!(peer_connection_state, Some(PeerConnectionState::Offline))
                })
                .collect::<HashMap<_, _>>();

            let timeout_duration = Duration::from_secs(5);
            
            if offline_peers.is_empty() {
                let response = WaitForPeersOnlineResponse::WaitForPeersOnlineSuccess {};
                let _ignore = tx_outbound.send(response.into())
                    .await
                    .inspect_err(|cause| warn!("Failed to send response:\n  {cause}"));
            } else {
                loop {
                    let received_peer_connection_state = tokio::time::timeout(timeout_duration, peer_state_subscription.receive()).await;
                    if tx_outbound.is_closed() {
                        debug!("Observer of peers <{peer_ids:?}> has closed the connection.");
                        break;
                    }
                    match received_peer_connection_state {
                        Ok(peer_connection_state) => {

                            if let Ok(SubscriptionEvent::Inserted { id: the_peer_id, value: PeerConnectionState::Online { remote_host } }) = peer_connection_state {
                                if offline_peers.contains_key(&the_peer_id) {
                                    offline_peers.remove(&the_peer_id);
                                    debug!("Observed peer <{the_peer_id}> is now online with remote address <{remote_host}>.");

                                    if offline_peers.is_empty() {
                                        let response = WaitForPeersOnlineResponse::WaitForPeersOnlineSuccess {};
                                        let _ignore = tx_outbound.send(response.into())
                                            .await
                                            .inspect_err(|cause| warn!("Failed to send response:\n  {cause}"));
                                        break;
                                    } else {
                                        let response = WaitForPeersOnlineResponse::WaitForPeersOnlinePending {};
                                        let _ignore = tx_outbound.send(response.into())
                                            .await
                                            .inspect_err(|cause| warn!("Failed to send response:\n  {cause}"));
                                    }
                                } else {
                                    trace!("Another peer with id <{the_peer_id}> is now online with remote address <{remote_host}>.");
                                }
                            }

                        }
                        Err(error) => {
                            let response = WaitForPeersOnlineResponse::WaitForPeersOnlinePending {};
                            let _ignore = tx_outbound.send(response.into())
                                .await
                                .inspect_err(|cause| warn!("Failed to send response:\n  {cause}"));

                            trace!("Observer of peer <{peer_ids:?}> did not get any updates for {}ms. {}", timeout_duration.as_millis(), error);
                        }
                    }
                }
            }
            // rx_outbound.close()

            trace!("Wait for observer to close connection.");
            let result = tokio::time::timeout(Duration::from_secs(10), tx_outbound.closed()).await;
            match result {
                Ok(_) => {
                    trace!("Observer channel flushed successfully.");
                }
                Err(elapsed) => {
                    trace!("Observer channel did not close in time. Elapsed: {:?}", elapsed);
                }
            }
            trace!("Leaving observation thread.");
        });

        Ok(rx_outbound)
    }
}
