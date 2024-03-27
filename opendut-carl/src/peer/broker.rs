use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tracing::{error, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use opendut_carl_api::proto::services::peer_messaging_broker::{ApplyPeerConfiguration, downstream, Downstream, TracingContext};
use opendut_carl_api::proto::services::peer_messaging_broker::Pong;
use opendut_carl_api::proto::services::peer_messaging_broker::upstream;
use opendut_types::peer::PeerId;
use opendut_types::peer::configuration::PeerConfiguration;
use opendut_types::peer::state::{PeerState, PeerUpState};

use crate::resources::manager::ResourcesManagerRef;

pub type PeerMessagingBrokerRef = Arc<PeerMessagingBroker>;


pub struct PeerMessagingBroker {
    resources_manager: ResourcesManagerRef,
    peers: Arc<RwLock<HashMap<PeerId, PeerMessagingRef>>>,
    options: PeerMessagingBrokerOptions,
}
struct PeerMessagingRef {
    downstream: mpsc::Sender<Downstream>,
}

impl PeerMessagingBroker {
    pub fn new(resources_manager: ResourcesManagerRef, options: PeerMessagingBrokerOptions) -> PeerMessagingBrokerRef {
        Arc::new(Self {
            resources_manager,
            peers: Default::default(),
            options,
        })
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn send_to_peer(&self, peer_id: PeerId, message: downstream::Message) -> Result<(), Error> {
        let downstream = {
            let peers = self.peers.read().await;
            peers.get(&peer_id)
                .map(|peer| &peer.downstream)
                .cloned()
        };
        let downstream = downstream.ok_or(Error::PeerNotFound(peer_id))?;

        let context = {
            let mut context = TracingContext { values: Default::default() };
            let propagator = TraceContextPropagator::new();
            let span = Span::current().entered();
            propagator.inject_context(&span.context(), &mut context.values);
            Some(context)
        };

        downstream.send(Downstream {
            context,
            message: Some(message)
        }).await.map_err(Error::DownstreamSend)?;
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
    ) -> (mpsc::Sender<upstream::Message>, mpsc::Receiver<Downstream>) {

        let (tx_inbound, mut rx_inbound) = mpsc::channel::<upstream::Message>(1024);
        let (tx_outbound, rx_outbound) = mpsc::channel::<Downstream>(1024);

        let peer_messaging_ref = PeerMessagingRef {
            downstream: Clone::clone(&tx_outbound),
        };

        self.peers.write().await.insert(peer_id, peer_messaging_ref);

        fn new_peer_up_state(remote_host: IpAddr) -> PeerState {
            PeerState::Up { inner: PeerUpState::Available, remote_host }
        }

        self.resources_manager.resources_mut(|resources| {
            resources.update::<PeerState>(peer_id)
                .modify(|peer_state| match peer_state {
                    PeerState::Up { inner: _, remote_host: peer_remote_host } => {
                        *peer_remote_host = remote_host
                    }
                    PeerState::Down => {
                        *peer_state = new_peer_up_state(remote_host)
                    }
                })
                .or_insert(new_peer_up_state(remote_host))
        }).await;

        if let Some(configuration) = self.resources_manager.get::<PeerConfiguration>(peer_id).await {
            if let Err(error) = self.send_to_peer(peer_id, downstream::Message::ApplyPeerConfiguration(
                ApplyPeerConfiguration {
                    configuration: Some(configuration.into())
                }
            )).await {
                error!("Failed to send ApplyPeerConfiguration message: {error}")
            };
        } else {
            error!("Failed to send ApplyPeerConfiguration message, because no PeerConfiguration found for peer: {peer_id}")
        }

        let timeout_duration = self.options.peer_disconnect_timeout;

        {
            let peers = Arc::clone(&self.peers);
            let resources_manager = Arc::clone(&self.resources_manager);

            tokio::spawn(async move {
                loop {
                    let received = tokio::time::timeout(timeout_duration, rx_inbound.recv()).await;

                    match received {
                        Ok(Some(message)) => handle_stream_message(message, peer_id, &tx_outbound).await,
                        Ok(None) => {
                            log::info!("Peer <{peer_id}> disconnected!");
                            break;
                        }
                        Err(_) => {
                            log::error!("No message from peer <{peer_id}> within {} ms.", timeout_duration.as_millis());
                            break;
                        }
                    }
                }
                down_peer_impl(resources_manager, peer_id).await;

                log::debug!("Removing peer <{peer_id}> from list of peers connected to message broker.");
                let mut peers = peers.write().await;
                let removed = peers.remove(&peer_id);
                if removed.is_none() {
                    log::error!("Failed to remove peer from list of peers connected to message broker.")
                }
            });
        }

        (tx_inbound, rx_outbound)
    }

    pub async fn remove_peer(&self, peer_id: PeerId) -> Result<(), Error> {
        let mut peers = self.peers.write().await;

        down_peer_impl(Arc::clone(&self.resources_manager), peer_id).await;

        match peers.remove(&peer_id) {
            Some(_) => Ok(()),
            None => Err(Error::PeerNotFound(peer_id)),
        }
    }
}

async fn handle_stream_message(
    message: upstream::Message,
    peer_id: PeerId,
    tx_outbound: &Sender<Downstream>,
) {
    match message {
        upstream::Message::Ping(_) => {
            let message = downstream::Message::Pong(Pong {});
            let context = None;
            let _ignore_result =
                tx_outbound.send(Downstream{message:Some(message), context}).await
                    .inspect_err(|cause| log::warn!("Failed to send ping to peer <{peer_id}>: {cause}"));
        },
    }
}

async fn down_peer_impl(resources_manager: ResourcesManagerRef, peer_id: PeerId) {
    log::debug!("Setting state of peer <{peer_id}> to Down.");

    resources_manager.resources_mut(|resources| {
        resources.update::<PeerState>(peer_id)
            .modify(|peer_state| {
                *peer_state = PeerState::Down;
            })
            .or_insert(PeerState::Down)
    }).await;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DownstreamSend Error: {0}")]
    DownstreamSend(SendError<Downstream>),
    #[error("PeerNotFound Error: {0}")]
    PeerNotFound(PeerId),
    #[error("Other Error: {message}")]
    Other { message: String },
}

#[derive(Clone)]
pub struct PeerMessagingBrokerOptions {
    pub peer_disconnect_timeout: Duration,
}
impl PeerMessagingBrokerOptions {
    pub fn load(config: &config::Config) -> Result<Self, opendut_util::settings::LoadError> {
        let peer_disconnect_timeout = Duration::from_millis(
            config.get::<u64>("peer.disconnect.timeout.ms")?
        );

        Ok(PeerMessagingBrokerOptions {
            peer_disconnect_timeout,
        })
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use googletest::prelude::*;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::Receiver;

    use opendut_carl_api::proto::services::peer_messaging_broker::Ping;

    use crate::resources::manager::ResourcesManager;

    use super::*;

    #[tokio::test]
    async fn peer_stream() -> Result<()> {
        let resources_manager = ResourcesManager::new();

        let options = PeerMessagingBrokerOptions {
            peer_disconnect_timeout: Duration::from_millis(200),
        };
        let testee = PeerMessagingBroker::new(Arc::clone(&resources_manager), options.clone());

        let peer_id = PeerId::random();
        let remote_host = IpAddr::from_str("1.2.3.4")?;

        let (sender, mut receiver) = testee.open(peer_id, remote_host).await;

        { //assert state contains peer connected and up
            let peers = testee.peers.read().await;

            assert!(peers.get(&peer_id).is_some());

            let peer_state = resources_manager.resources(|resources| {
                resources.get::<PeerState>(peer_id)
            }).await;
            let peer_state = peer_state.expect("PeerState for peer <{peer_id}> should exist.");
            match peer_state {
                PeerState::Up { .. } => {} //Success
                _ => {
                    fail!("PeerState should be 'Up'.")?;
                }
            }
        }

        { //test repeated pings
            for _ in 1..5 {
                tokio::time::sleep(
                    options.peer_disconnect_timeout / 2 //less than timeout
                ).await;

                do_ping(&sender, &mut receiver).await;
            }
        }

        { //assert state contains peer disconnected and down after missing pings
            tokio::time::sleep(
                options.peer_disconnect_timeout * 2 //more than timeout
            ).await;

            let peers = testee.peers.read().await;

            assert!(peers.get(&peer_id).is_none());

            let peer_state = resources_manager.resources(|resources| {
                resources.get::<PeerState>(peer_id)
            }).await;
            let peer_state = peer_state.expect("PeerState for peer <{peer_id}> should exist.");
            match peer_state {
                PeerState::Down { .. } => {} //Success
                _ => {
                    fail!("PeerState should be 'Down' after timeout.")?;
                }
            }
        }

        Ok(())
    }

    async fn do_ping(sender: &mpsc::Sender<upstream::Message>, receiver: &mut Receiver<Downstream>) {
        sender.send(upstream::Message::Ping(Ping {})).await
            .unwrap();

        let received = receiver.recv().await.unwrap();

        assert_eq!(received.message, Some(downstream::Message::Pong(Pong {})));
    }
}
