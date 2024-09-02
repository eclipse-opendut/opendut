use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::error::SendError;
use tracing::{debug, error, info, Span, warn};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use opendut_carl_api::proto::services::peer_messaging_broker::{ApplyPeerConfiguration, downstream, Downstream, TracingContext};
use opendut_carl_api::proto::services::peer_messaging_broker::Pong;
use opendut_carl_api::proto::services::peer_messaging_broker::upstream;
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration2};
use opendut_types::peer::PeerId;
use opendut_types::peer::state::{PeerState, PeerUpState};

use crate::persistence::error::PersistenceError;
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

    /// deprecated since version 0.2.0
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
    ) -> Result<(mpsc::Sender<upstream::Message>, mpsc::Receiver<Downstream>), OpenError> {

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
            let maybe_peer_state = resources.get::<PeerState>(peer_id)
                .map_err(|source| OpenError::Persistence { peer_id, source })?;
                
            match maybe_peer_state {
                None => {
                    info!("Peer <{}> opened stream which has not been seen before.", peer_id);
                    Ok(new_peer_up_state(remote_host))
                }
                Some(peer_state) => match peer_state {
                    PeerState::Down => {
                        debug!("Peer <{peer_id}> opened stream which was previously down.");
                        Ok(new_peer_up_state(remote_host))
                    }
                    PeerState::Up { .. } => {
                        error!("Peer <{peer_id}> opened stream which was already connected.");
                        Err(OpenError::PeerAlreadyConnected { peer_id })
                    }
                }
            }
            .and_then(|new_peer_state| {
                resources.insert(peer_id, new_peer_state)
                    .map_err(|source| OpenError::Persistence { peer_id, source })
            })
        }).await?;

        let maybe_old_configuration = self.resources_manager.get::<OldPeerConfiguration>(peer_id).await
            .map_err(|source| OpenError::Persistence { peer_id, source })?;
        let maybe_configuration2 = self.resources_manager.get::<PeerConfiguration2>(peer_id).await
            .map_err(|source| OpenError::Persistence { peer_id, source })?;

        if let Some(old_configuration) = maybe_old_configuration {
            if let Some(configuration2) = maybe_configuration2 {

                self.send_to_peer(peer_id, downstream::Message::ApplyPeerConfiguration(
                    ApplyPeerConfiguration {
                        old_configuration: Some(old_configuration.into()),
                        configuration2: Some(configuration2.into()),
                    }
                )).await
                .map_err(|cause| OpenError::SendApplyPeerConfiguration { peer_id, cause: cause.to_string() })?;
            }
            else {
                return Err(OpenError::SendApplyPeerConfiguration { peer_id, cause: String::from("No PeerConfiguration2 found for peer") });
            }
        } else {
            return Err(OpenError::SendApplyPeerConfiguration { peer_id, cause: String::from("No OldPeerConfiguration found for peer") });
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
                            info!("Peer <{peer_id}> disconnected!");
                            break;
                        }
                        Err(cause) => {
                            error!("No message from peer <{peer_id}> within {} ms: {cause}", timeout_duration.as_millis());
                            break;
                        }
                    }
                }
                Self::remove_peer_impl(peer_id, resources_manager, peers).await
                    .unwrap_or_else(|cause| error!("Error while removing peer after its stream ended: {cause}"));
            });
        }

        Ok((tx_inbound, rx_outbound))
    }

    pub async fn remove_peer(&self, peer_id: PeerId) -> Result<(), RemovePeerError> {
        Self::remove_peer_impl(peer_id, Arc::clone(&self.resources_manager), Arc::clone(&self.peers)).await
    }

    async fn remove_peer_impl(
        peer_id: PeerId,
        resources_manager: ResourcesManagerRef,
        peers: Arc<RwLock<HashMap<PeerId, PeerMessagingRef>>>
    ) -> Result<(), RemovePeerError> {
        debug!("Setting state of peer <{peer_id}> to Down.");
        resources_manager.insert(peer_id, PeerState::Down).await
            .map_err(|source| RemovePeerError::Persistence { peer_id, source })?;

        debug!("Removing peer <{peer_id}> from list of peers connected to message broker.");
        let mut peers = peers.write().await;
        match peers.remove(&peer_id) {
            Some(_) => Ok(()),
            None => Err(RemovePeerError::PeerNotFound(peer_id)),
        }
    }
}

async fn handle_stream_message(
    message: upstream::Message,
    peer_id: PeerId,
    tx_outbound: &mpsc::Sender<Downstream>,
) {
    match message {
        upstream::Message::Ping(_) => {
            let message = downstream::Message::Pong(Pong {});
            let context = None;
            let _ignore_result =
                tx_outbound.send(Downstream{message:Some(message), context}).await
                    .inspect_err(|cause| warn!("Failed to send ping to peer <{peer_id}>: {cause}"));
        },
    }
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

#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    #[error(
        "Peer <{peer_id}> opened stream, but CARL already has a connected stream with this PeerId. \
        This likely means that someone set up a second host using the same PeerId. \
        Rejecting connection."
    )]
    PeerAlreadyConnected { peer_id: PeerId },

    #[error("Error while sending peer configuration to peer:\n  {cause}")]
    SendApplyPeerConfiguration { peer_id: PeerId, cause: String },

    #[error("Error while accessing persistence after Peer <{peer_id}> opened stream.")]
    Persistence { peer_id: PeerId, #[source] source: PersistenceError },
}

#[derive(Debug, thiserror::Error)]
pub enum RemovePeerError {
    #[error("PeerNotFound Error while removing peer: {0}")]
    PeerNotFound(PeerId),
    #[error("Error while accessing persistence when removing Peer <{peer_id}>.")]
    Persistence { peer_id: PeerId, #[source] source: PersistenceError },
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
    use opendut_types::peer::configuration::PeerNetworkConfiguration;
    use opendut_types::util::net::NetworkInterfaceName;

    use crate::resources::manager::ResourcesManager;

    use super::*;

    #[tokio::test]
    async fn peer_stream() -> anyhow::Result<()> {
        let Fixture { resources_manager, peer_id } = fixture().await?;

        let options = PeerMessagingBrokerOptions {
            peer_disconnect_timeout: Duration::from_millis(200),
        };
        let testee = PeerMessagingBroker::new(Arc::clone(&resources_manager), options.clone());

        let remote_host = IpAddr::from_str("1.2.3.4")?;

        let (sender, mut receiver) = testee.open(peer_id, remote_host).await?;

        { //assert state contains peer connected and up
            let peers = testee.peers.read().await;

            assert!(peers.get(&peer_id).is_some());

            let peer_state = resources_manager.resources(|resources| {
                resources.get::<PeerState>(peer_id)
            }).await?;
            let peer_state = peer_state.expect("PeerState for peer <{peer_id}> should exist.");
            match peer_state {
                PeerState::Up { .. } => {} //Success
                _ => {
                    panic!("PeerState should be 'Up'.");
                }
            }
        }

        { //Receive initial ApplyPeerConfiguration
            let received = receiver.recv().await.unwrap().message.unwrap();

            assert_that!(
                received,
                matches_pattern!(
                    downstream::Message::ApplyPeerConfiguration(
                        matches_pattern!(ApplyPeerConfiguration { .. })
                    )
                )
            );
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
            }).await?;
            let peer_state = peer_state.expect("PeerState for peer <{peer_id}> should exist.");
            match peer_state {
                PeerState::Down { .. } => {} //Success
                _ => {
                    panic!("PeerState should be 'Down' after timeout.");
                }
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_reject_second_connection_for_peer() -> anyhow::Result<()> {
        let Fixture { resources_manager, peer_id } = fixture().await?;

        let options = PeerMessagingBrokerOptions {
            peer_disconnect_timeout: Duration::from_millis(200),
        };
        let testee = PeerMessagingBroker::new(Arc::clone(&resources_manager), options.clone());

        let remote_host = IpAddr::from_str("1.2.3.4")?;

        let result = testee.open(peer_id, remote_host).await;
        assert!(result.is_ok());

        let result = testee.open(peer_id, remote_host).await;
        assert_that!(
            result.unwrap_err(),
            matches_pattern!(OpenError::PeerAlreadyConnected { peer_id: eq(peer_id) })
        );

        Ok(())
    }

    async fn do_ping(sender: &mpsc::Sender<upstream::Message>, receiver: &mut Receiver<Downstream>) {
        sender.send(upstream::Message::Ping(Ping {})).await
            .unwrap();

        let received = receiver.recv().await.unwrap();

        assert_eq!(received.message, Some(downstream::Message::Pong(Pong {})));
    }

    struct Fixture {
        resources_manager: ResourcesManagerRef,
        peer_id: PeerId,
    }
    async fn fixture() -> anyhow::Result<Fixture> {
        let resources_manager = ResourcesManager::new_in_memory();

        let peer_id = PeerId::random();
        resources_manager.insert(peer_id, OldPeerConfiguration {
            cluster_assignment: None,
            network: PeerNetworkConfiguration {
                bridge_name: NetworkInterfaceName::try_from("br0")?,
            },
        }).await?;
        resources_manager.insert(peer_id, PeerConfiguration2 {
            executors: vec![],
        }).await?;

        Ok(Fixture {
            resources_manager,
            peer_id,
        })
    }
}
