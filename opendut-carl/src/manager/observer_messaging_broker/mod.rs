mod observe_peers_online;

use crate::manager::cluster_manager::ClusterManagerRef;
use crate::manager::observer_messaging_broker::observe_peers_online::ObservePeersOnline;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::PersistenceError;
use opendut_carl_api::carl::observer::{WaitForPeersOnlineResponseStatus};
use opendut_types::peer::PeerId;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, trace, warn};

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
    
    pub async fn wait_for_peers_online(&self, peer_ids: HashSet<PeerId>, max_observation_duration: Duration) -> Result<mpsc::Receiver<opendut_carl_api::proto::services::observer_messaging_broker::WaitForPeersOnlineResponse>, ObserverError> {
        let (tx_outbound, rx_outbound) = mpsc::channel::<opendut_carl_api::proto::services::observer_messaging_broker::WaitForPeersOnlineResponse>(1024);
        let timeout_duration = Duration::from_secs(5);
        let mut observed_peer_connection_states = ObservePeersOnline::create(peer_ids.clone(), self.resource_manager.clone(), timeout_duration, max_observation_duration).await?;

        tokio::spawn(async move {
            let response = observed_peer_connection_states.determine_response();
            if let WaitForPeersOnlineResponseStatus::WaitForPeersOnlineSuccess = response.status {
                let _ignore = tx_outbound.send(response.into())
                    .await
                    .inspect_err(|cause| warn!("Failed to send response:\n  {cause}"));
            } else {
                loop {
                    observed_peer_connection_states.observe().await;
                    if tx_outbound.is_closed() {
                        debug!("Observer of peers <{peer_ids:?}> has closed the connection.");
                        break;
                    }
                    let response = observed_peer_connection_states.determine_response();
                    let _ignore = tx_outbound.send(response.clone().into())
                        .await
                        .inspect_err(|cause| warn!("Failed to send response:\n  {cause}"));
                    if let WaitForPeersOnlineResponseStatus::WaitForPeersOnlineSuccess = response.status {
                        break;
                    }
                }
            }

            trace!("Wait for observer to close connection.");
            let _ = tokio::time::timeout(Duration::from_secs(10), tx_outbound.closed()).await
                .inspect_err(|elapsed| trace!("Observer channel did not close in time. Elapsed: {:?}", elapsed));
            trace!("Leaving observation thread.");
        });

        Ok(rx_outbound)
    }
}
