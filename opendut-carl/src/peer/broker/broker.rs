use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use opendut_carl_api::proto::services::peer_messaging_broker::downstream;
use opendut_carl_api::proto::services::peer_messaging_broker::Pong;
use opendut_carl_api::proto::services::peer_messaging_broker::upstream;
use opendut_types::peer::PeerId;

use super::error::{Error, Result};

pub type PeerMessagingBrokerRef = Arc<PeerMessagingBroker>;

#[derive(Clone)]
pub struct PeerMessagingBroker {
    peers: Arc<RwLock<HashMap<PeerId, PeerState>>>,
}
struct PeerState {
    downstream: mpsc::Sender<downstream::Message>,
}


impl PeerMessagingBroker {
    pub fn new() -> Self {
        Self {
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
    ) -> (mpsc::Sender<upstream::Message>, mpsc::Receiver<downstream::Message>) {

        let (tx_inbound, mut rx_inbound) = mpsc::channel::<upstream::Message>(1024);
        let (tx_outbound, rx_outbound) = mpsc::channel::<downstream::Message>(1024);

        self.peers.write().await.insert(
            peer_id,
            PeerState {
                downstream: Clone::clone(&tx_outbound),
            }
        );

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
