use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::error::SendError;

use opendut_carl_api::proto::services::peer_messaging_broker::downstream;
use opendut_carl_api::proto::services::peer_messaging_broker::Pong;
use opendut_carl_api::proto::services::peer_messaging_broker::upstream;
use opendut_types::peer::PeerId;
use opendut_types::peer::state::{PeerState, PeerUpState};

use crate::resources::manager::ResourcesManagerRef;

pub type PeerMessagingBrokerRef = Arc<PeerMessagingBroker>;


#[derive(Clone)]
pub struct PeerMessagingBroker {
    resources_manager: ResourcesManagerRef,
    peers: Arc<RwLock<HashMap<PeerId, PeerMessagingRef>>>,
}
struct PeerMessagingRef {
    downstream: mpsc::Sender<downstream::Message>,
}


impl PeerMessagingBroker {
    pub fn new(resources_manager: ResourcesManagerRef) -> Self {
        Self {
            resources_manager,
            peers: Default::default(),
        }
    }

    pub async fn send_to_peer(&self, peer_id: PeerId, message: downstream::Message) -> Result<()> {
        let downstream = {
            let peers = self.peers.read().await;
            peers.get(&peer_id).map(|peer| Clone::clone(&peer.downstream))
        };
        let downstream = downstream.ok_or(Error::PeerNotFound(peer_id))?;

        downstream.send(message).await.map_err(Error::DownstreamSend)?;
        Ok(())
    }

    pub async fn list_peers(&self) -> Vec<PeerId> {
        let peers = self.peers.read().await;

        peers.keys()
            .cloned()
            .collect::<Vec<_>>()
    }

    pub async fn open(
        &self,
        peer_id: PeerId,
        remote_host: IpAddr,
    ) -> (mpsc::Sender<upstream::Message>, mpsc::Receiver<downstream::Message>) {

        let (tx_inbound, mut rx_inbound) = mpsc::channel::<upstream::Message>(1024);
        let (tx_outbound, rx_outbound) = mpsc::channel::<downstream::Message>(1024);

        let peer_messaging_ref = PeerMessagingRef {
            downstream: Clone::clone(&tx_outbound),
        };

        self.peers.write().await.insert(peer_id, peer_messaging_ref);

        fn new_peer_state(remote_host: IpAddr) -> PeerState {
            PeerState::Up { inner: PeerUpState::Available, remote_host }
        }

        self.resources_manager.resources_mut(|resources| {
            resources.update::<PeerState>(peer_id)
                .modify(|peer_state| match peer_state {
                    PeerState::Up { inner: _, remote_host: peer_remote_host } => {
                        *peer_remote_host = remote_host
                    }
                    PeerState::Down => {
                        *peer_state = new_peer_state(remote_host)
                    }
                })
                .or_insert(new_peer_state(remote_host))
        }).await;


        tokio::spawn(async move {
            while let Some(message) = rx_inbound.recv().await {
                let downstream = match message {
                    upstream::Message::Ping(_) => downstream::Message::Pong(Pong {}),
                };
                tx_outbound.send(downstream).await.unwrap();
            }
        });

        (tx_inbound, rx_outbound)
    }

    pub async fn remove_peer(&self, peer_id: PeerId) -> Result<()> {
        let mut peers = self.peers.write().await;

        match peers.remove(&peer_id) {
            Some(_) => Ok(()),
            None => Err(Error::PeerNotFound(peer_id)),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DownstreamSend Error: {0}")]
    DownstreamSend(SendError<downstream::Message>),
    #[error("PeerNotFound Error: {0}")]
    PeerNotFound(PeerId),
    #[error("Other Error: {message}")]
    Other { message: String },
}
pub type Result<T> = std::result::Result<T, Error>;
