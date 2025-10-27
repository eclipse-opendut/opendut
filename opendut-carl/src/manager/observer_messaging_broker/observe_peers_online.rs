use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::time::{Duration, Instant};
use tracing::{debug, error, trace};
use opendut_carl_api::carl::observer::{WaitForPeersOnlineResponse, WaitForPeersOnlineResponseStatus};
use opendut_model::peer::PeerId;
use opendut_model::peer::state::PeerConnectionState;
use crate::resource::manager::{ResourceManagerRef, SubscriptionEvent};
use crate::resource::persistence::error::PersistenceError;
use crate::resource::subscription::Subscription;

pub struct ObservePeersOnline {
    peers: HashMap<PeerId, PeerConnectionState>,
    peer_connection_state_subscription: Subscription<PeerConnectionState>,
    timeout: Duration,
    max_observation_duration: Duration,
    observation_start: Instant,
}

impl ObservePeersOnline {
    pub async fn create(peer_ids: HashSet<PeerId>, resource_manager: ResourceManagerRef, timeout: Duration, max_observation_duration: Duration) -> Result<Self, PersistenceError> {
        let all_peer_connection_states = resource_manager.list::<PeerConnectionState>().await?;

        let selected_peer_connection_states = peer_ids.iter().map(|peer_id| {
            let peer_connection_state = all_peer_connection_states.get(peer_id);
            match peer_connection_state {
                None => {
                    (*peer_id, PeerConnectionState::Offline)
                }
                Some(connection_state) => {
                    (*peer_id, connection_state.clone())
                }
            }
        }).collect::<HashMap<_, _>>();

        let peer_connection_state_subscription = resource_manager.subscribe::<PeerConnectionState>().await;

        Ok(Self {
            peers: selected_peer_connection_states,
            peer_connection_state_subscription,
            timeout,
            max_observation_duration,
            observation_start: Instant::now(),
        })
    }
    
    pub async fn observe(&mut self) {
        let received_peer_connection_state = tokio::time::timeout(self.timeout, self.peer_connection_state_subscription.receive()).await;
        match received_peer_connection_state {
            Ok(peer_connection_state) => {
                match peer_connection_state {
                    Ok(event) => {
                        match event {
                            SubscriptionEvent::Inserted { id, value } => {
                                match value {
                                    PeerConnectionState::Online { .. } => {
                                        self.peer_online(id, value);
                                    },
                                    PeerConnectionState::Offline => {
                                        self.peer_offline(id);
                                    }
                                }
                            }
                            SubscriptionEvent::Removed { id, .. } => {
                                self.peer_offline(id);
                            }
                        }
                    }
                    Err(error) => {
                        error!("Failed to observe peer online status. Subscription error: {error}");
                    }
                }

            }
            Err(error) => {
                let peer_ids = self.peers.keys().copied().collect::<HashSet<_>>();
                trace!("Observer of peer <{peer_ids:?}> did not get any updates for {}ms. {}", self.timeout.as_millis(), error);
            }
        }
    }
    
    fn offline_peers(&self) -> HashSet<PeerId> {
        self.peers.iter()
            .filter(|(_, peer_connection_state)| matches!(peer_connection_state, PeerConnectionState::Offline))
            .map(|(peer_id, _)| *peer_id)
            .collect::<HashSet<PeerId>>()        
    }
    
    fn peer_online(&mut self, peer_id: PeerId, connection_state: PeerConnectionState) {
        if let PeerConnectionState::Online { remote_host } = connection_state
        && let Entry::Occupied(mut entry) = self.peers.entry(peer_id) {
            debug!("Observed peer <{peer_id}> is now online with remote address <{remote_host}>.");
            entry.insert(connection_state);
        }
    }

    fn peer_offline(&mut self, peer_id: PeerId) {
        if let Entry::Occupied(mut entry) = self.peers.entry(peer_id) {
            debug!("Observed peer <{peer_id}> is now offline.");
            entry.insert(PeerConnectionState::Offline);            
        }
    }

    pub fn determine_response(&self) -> WaitForPeersOnlineResponse {
        let offline_peers = self.offline_peers();
        if offline_peers.is_empty() {
            WaitForPeersOnlineResponse {
                peers: self.peers.clone(),
                status: WaitForPeersOnlineResponseStatus::WaitForPeersOnlineSuccess,
            }
        } else {
            let now = Instant::now();
            let observation_duration = now.saturating_duration_since(self.observation_start);
            if observation_duration < self.max_observation_duration {
                WaitForPeersOnlineResponse {
                    peers: self.peers.clone(),
                    status: WaitForPeersOnlineResponseStatus::WaitForPeersOnlinePending,
                }
            } else {
                WaitForPeersOnlineResponse { 
                    peers: self.peers.clone(),
                    status: WaitForPeersOnlineResponseStatus::WaitForPeersOnlineFailure {
                        reason: "Maximum observation time".to_string(),
                    }
                }
            }
        }
    }
}
