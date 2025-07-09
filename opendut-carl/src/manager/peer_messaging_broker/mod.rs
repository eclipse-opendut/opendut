use std::collections::HashMap;
use std::net::IpAddr;
use std::ops::Not;
use std::sync::Arc;
use std::time::Duration;

use opendut_carl_api::carl::broker::stream_header;
use opendut_carl_api::proto::services::peer_messaging_broker::{upstream, DisconnectNotice};
use opendut_carl_api::proto::services::peer_messaging_broker::Pong;
use opendut_carl_api::proto::services::peer_messaging_broker::{downstream, ApplyPeerConfiguration, Downstream, TracingContext};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use opendut_types::peer::state::{PeerConnectionState};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, trace, warn, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use opendut_carl_api::carl::broker::stream_header::PeerVersion;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::storage::ResourcesStorageApi;

mod effects;

pub type PeerMessagingBrokerRef = Arc<PeerMessagingBroker>;


pub struct PeerMessagingBroker {
    resource_manager: ResourceManagerRef,
    peers: Arc<RwLock<HashMap<PeerId, PeerMessagingRef>>>,
    options: PeerMessagingBrokerOptions,
}
struct PeerMessagingRef {
    downstream: mpsc::Sender<Downstream>,
    disconnected: bool,
}

impl PeerMessagingBroker {
    pub async fn new(resource_manager: ResourceManagerRef, options: PeerMessagingBrokerOptions) -> PeerMessagingBrokerRef {
        let self_ref = Arc::new(Self {
            resource_manager: resource_manager.clone(),
            peers: Default::default(),
            options,
        });
        effects::register(resource_manager, self_ref.clone()).await;

        self_ref
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
        }).await.map_err(|error| Error::DownstreamSend(Box::new(error)))?;
        Ok(())
    }

    async fn disconnect(&self, peer_id: PeerId) -> Result<(), Error> {
        let downstream = {
            let peers = self.peers.read().await;
            peers.get(&peer_id)
                .map(|peer| &peer.downstream)
                .cloned()
        };
        let downstream = downstream.ok_or(Error::PeerNotFound(peer_id))?;
        let disconnect_message = downstream::Message::DisconnectNotice(DisconnectNotice { });

        downstream.send(Downstream {
            context: None,
            message: Some(disconnect_message)
        }).await.map_err(|error| Error::DownstreamSend(Box::new(error)))?;

        let peer_messaging_ref = PeerMessagingRef {
            downstream,
            disconnected: true,
        };

        self.peers.write().await.insert(peer_id, peer_messaging_ref);

        Ok(())
    }

    pub async fn remove_peer(&self, peer_id: PeerId) -> Result<(), RemovePeerError> {
        let peer_connection_state = self.resource_manager.get::<PeerConnectionState>(peer_id).await;
        if let Ok(Some(peer_connection_state)) = peer_connection_state {
            match peer_connection_state {
                PeerConnectionState::Offline => {
                    debug!("Removing connection state of peer <{peer_id}> since the peer descriptor itself was deleted.");
                    let _ = self.resource_manager.remove::<PeerConnectionState>(peer_id).await;
                }
                PeerConnectionState::Online { .. } => {
                    let result = self.disconnect(peer_id).await;
                    match result {
                        Ok(_) => {
                            tracing::log::trace!("Sent disconnect notice to online peer <{peer_id}>.");
                        }
                        Err(_) => {
                            error!("Failed to send disconnect notice to online peer <{peer_id}>.");
                        }
                    }
                }
            }
        }
        Ok(())
    }


    pub async fn open(
        &self,
        peer_id: PeerId,
        remote_host: IpAddr,
        extra_headers: stream_header::ExtraHeaders,
    ) -> Result<(mpsc::Sender<upstream::Message>, mpsc::Receiver<Downstream>), OpenError> {

        debug!("Peer <{peer_id}> opened stream from remote address {remote_host} with extra headers: {extra_headers:?}");
        log_version_compatibility(peer_id, remote_host, extra_headers.client_version)
            .inspect_err(|error| warn!("Failed to check version compatibility with newly connected peer <{peer_id}>: {error}"))
            .ok();

        let (tx_inbound, mut rx_inbound) = mpsc::channel::<upstream::Message>(1024);
        let (tx_outbound, rx_outbound) = mpsc::channel::<Downstream>(1024);

        let peer_messaging_ref = PeerMessagingRef {
            downstream: Clone::clone(&tx_outbound),
            disconnected: false,
        };

        self.expect_known_peer_descriptor(peer_id).await?.ok_or(OpenError::PeerNotFound(peer_id))?;
        self.update_peer_connection_state(peer_id, remote_host).await?;
        self.peers.write().await.insert(peer_id, peer_messaging_ref);
        self.send_initial_peer_configuration(peer_id).await?;

        let timeout_duration = self.options.peer_disconnect_timeout;

        {
            let peers = Arc::clone(&self.peers);
            let resource_manager = Arc::clone(&self.resource_manager);

            tokio::spawn(async move {
                loop {
                    let received = tokio::time::timeout(timeout_duration, rx_inbound.recv()).await;
                    match received {
                        Ok(Some(message)) => handle_stream_message(message, peer_id, &tx_outbound).await,
                        Ok(None) => {
                            info!("Peer <{peer_id}> disconnected! Closing inbound channel.");
                            break;
                        }
                        Err(cause) => {
                            error!("No message from peer <{peer_id}> within {} ms:\n  {cause}. Closing connection.", timeout_duration.as_millis());
                            rx_inbound.close();
                        }
                    }

                    let downstream_disconnected = peers.read().await.get(&peer_id)
                        .map(|peer| &peer.disconnected).cloned()
                        .unwrap_or_default();
                    if downstream_disconnected {
                        info!("Peer <{peer_id}> shall be disconnected!");
                        rx_inbound.close();
                    }
                }
                let channel_close_grace_time = tokio::time::timeout(timeout_duration, tx_outbound.closed()).await;
                match channel_close_grace_time {
                    Ok(_) => {
                        trace!("Peer channel flushed successfully.");
                    }
                    Err(_) => {
                        trace!("Peer channel did not close voluntarily in time. Closing anyway.");
                    }
                }

                Self::remove_peer_impl(peer_id, resource_manager, peers).await
                    .unwrap_or_else(|cause| error!("Error while removing peer after its stream ended:\n  {cause}"));
            });
        }

        Ok((tx_inbound, rx_outbound))
    }
    
    async fn send_initial_peer_configuration(&self, peer_id: PeerId) -> Result<(), OpenError> {
        let old_peer_configuration = self.resource_manager.get::<OldPeerConfiguration>(peer_id).await
            .map_err(|source| OpenError::Persistence { peer_id, source })?;
        let old_peer_configuration = match old_peer_configuration {
            Some(old_peer_configuration) => {
                debug!("Found an OldPeerConfiguration for newly connected peer <{peer_id}>. Re-sending this configuration:\n{old_peer_configuration:#?}");
                old_peer_configuration
            }
            None => {
                //OldPeerConfiguration is not persisted across CARL restarts
                debug!("No OldPeerConfiguration found for newly connected peer <{peer_id}>. Sending empty configuration.");
                OldPeerConfiguration::default()
            }
        };

        let peer_configuration = self.resource_manager.get::<PeerConfiguration>(peer_id).await
            .map_err(|source| OpenError::Persistence { peer_id, source })?;
        let peer_configuration = match peer_configuration {
            Some(peer_configuration) => {
                debug!("Found a PeerConfiguration for newly connected peer <{peer_id}>. Re-sending this configuration.\n{peer_configuration:#?}");
                peer_configuration
            }
            None => {
                //PeerConfiguration is not persisted across CARL restarts
                debug!("No PeerConfiguration found for newly connected peer <{peer_id}>. Sending empty configuration.");
                PeerConfiguration::default()
            }
        };

        self.send_to_peer(peer_id, downstream::Message::ApplyPeerConfiguration(
            ApplyPeerConfiguration {
                old_configuration: Some(old_peer_configuration.into()),
                configuration: Some(peer_configuration.into()),
            }
        )).await
            .map_err(|cause| OpenError::SendApplyPeerConfiguration { peer_id, cause: cause.to_string() })?;
        Ok(())
    }

    async fn expect_known_peer_descriptor(&self, peer_id: PeerId) -> Result<Option<PeerDescriptor>, OpenError> {
        self.resource_manager.resources_mut(async |resources| {
            resources.get::<PeerDescriptor>(peer_id)
                .map_err(|source| OpenError::Persistence { peer_id, source })
        }).await
            .map_err(|source| OpenError::Persistence { peer_id, source })?
    }

    async fn update_peer_connection_state(&self, peer_id: PeerId, remote_host: IpAddr) -> Result<(), OpenError> {
        self.resource_manager.resources_mut(async |resources| {
            let maybe_peer_state = resources.get::<PeerConnectionState>(peer_id)
                .map_err(|source| OpenError::Persistence { peer_id, source })?;

            match maybe_peer_state {
                None => {
                    info!("Peer <{peer_id}> had not been seen before.");
                    Ok(PeerConnectionState::Online { remote_host })
                }
                Some(peer_connection_state) => match peer_connection_state {
                    PeerConnectionState::Offline => {
                        debug!("Peer <{peer_id}> had been seen before and was down.");
                        Ok(PeerConnectionState::Online { remote_host })
                    }
                    PeerConnectionState::Online { .. } => {
                        error!("Peer <{peer_id}> opened stream which was already connected. Rejecting.");
                        Err(OpenError::PeerAlreadyConnected { peer_id })
                    }
                }
            }
                .and_then(|new_peer_connection_state| {
                    resources.insert(peer_id, new_peer_connection_state)
                        .map_err(|source| OpenError::Persistence { peer_id, source })
                })
        }).await
            .map_err(|source| OpenError::Persistence { peer_id, source })??;

        Ok(())
    }

    async fn remove_peer_impl(
        peer_id: PeerId,
        resource_manager: ResourceManagerRef,
        peers: Arc<RwLock<HashMap<PeerId, PeerMessagingRef>>>
    ) -> Result<(), RemovePeerError> {
        let peer_connection_state = resource_manager.get::<PeerConnectionState>(peer_id).await
            .map_err(|source| RemovePeerError::Persistence { peer_id, source })?
            .ok_or(RemovePeerError::PeerNotFound(peer_id))?;

        let peer_descriptor_deleted = resource_manager.get::<PeerDescriptor>(peer_id).await
            .map_err(|source| RemovePeerError::Persistence { peer_id, source })?.is_none();
        if peer_descriptor_deleted {
            let _ = resource_manager.remove::<PeerConnectionState>(peer_id).await
                .map_err(|source| RemovePeerError::Persistence { peer_id, source })?;
        } else {
            debug!("Setting connection state of peer <{peer_id}> to offline.");
            resource_manager.insert(peer_id, PeerConnectionState::Offline).await
                .map_err(|source| RemovePeerError::Persistence { peer_id, source })?;
        }
        if let PeerConnectionState::Online { remote_host } = peer_connection_state {
            debug!("Removing peer <{peer_id}> from list of peers connected to message broker. Last known address <{remote_host}>.");
        } else {
            debug!("Removing peer <{peer_id}> from list of peers connected to message broker. No previously known address.");
        }
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
                tx_outbound.send(Downstream { message: Some(message), context }).await
                    .inspect_err(|cause| warn!("Failed to send ping to peer <{peer_id}>:\n  {cause}"));
        },
        upstream::Message::PeerConfigurationState(_) => {}
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DownstreamSend Error: {0}")]
    DownstreamSend(Box<SendError<Downstream>>),
    #[error("PeerNotFound Error: {0}")]
    PeerNotFound(PeerId),
}

#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    #[error(
        "Peer <{peer_id}> opened stream, but CARL already has a connected stream with this PeerId. \
        This likely means that someone set up a second host using the same PeerId. \
        Rejecting connection."
    )]
    PeerAlreadyConnected { peer_id: PeerId },
    #[error("Peer not found. Unknown peer id: <{0}>")]
    PeerNotFound(PeerId),

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

fn log_version_compatibility(
    peer_id: PeerId,
    remote_host: IpAddr,
    client_version: Option<PeerVersion>,
) -> anyhow::Result<()> {
    if let Some(client_version) = client_version {
        let client_version = semver::Version::parse(&client_version.value)?;
        let version_requirement = semver::VersionReq::parse(crate::app_info::PKG_VERSION)?;

        if version_requirement.matches(&client_version).not() {
            warn!("Peer <{peer_id}> newly connected from {remote_host} has incompatible version {client_version}. Should have version compatible with CARL's version ({version_requirement}).");
        }
    } else {
        warn!("Peer <{peer_id}> newly connected from {remote_host} did not send a client version. Cannot check version compatibility.");
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use googletest::prelude::*;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::Receiver;

    use opendut_carl_api::proto::services::peer_messaging_broker::Ping;
    use crate::manager::peer_manager::tests::create_peer_descriptor;
    use super::*;
    use crate::resource::manager::ResourceManager;
    use crate::resource::storage::ResourcesStorageApi;

    #[test_log::test(tokio::test)]
    async fn peer_stream() -> anyhow::Result<()> {
        let Fixture { resource_manager, peer_id } = fixture().await?;

        let options = PeerMessagingBrokerOptions {
            peer_disconnect_timeout: Duration::from_millis(200),
        };
        let testee = PeerMessagingBroker::new(Arc::clone(&resource_manager), options.clone()).await;

        let remote_host = IpAddr::from_str("1.2.3.4")?;

        let (sender, mut receiver) = testee.open(peer_id, remote_host, stream_header::ExtraHeaders::default()).await?;

        { //assert state contains peer connected and up
            let peers = testee.peers.read().await;

            assert!(peers.get(&peer_id).is_some());

            let peer_connection_state = resource_manager.resources(async |resources| {
                resources.get::<PeerConnectionState>(peer_id)
            }).await??;
            let peer_connection_state = peer_connection_state.unwrap_or_else(|| panic!("PeerConnectionState for peer <{peer_id}> should exist."));
            match peer_connection_state {
                PeerConnectionState::Online { .. } => {} //Success
                _ => {
                    panic!("PeerConnectionState should be 'Online'.");
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
                options.peer_disconnect_timeout * 6  // more than receive timeout + channel close timeout
            ).await;

            let peers = testee.peers.read().await;

            assert!(peers.get(&peer_id).is_none());

            let peer_connection_state = resource_manager.resources(async |resources| {
                resources.get::<PeerConnectionState>(peer_id)
            }).await??;
            let peer_connection_state = peer_connection_state.unwrap_or_else(|| panic!("PeerConnectionState for peer <{peer_id}> should exist."));
            match peer_connection_state {
                PeerConnectionState::Offline => {} //Success
                _ => {
                    panic!("PeerConnectionState should be 'Down' after timeout.");
                }
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn should_disconnect_peer_when_peer_descriptor_was_removed() -> anyhow::Result<()> {
        let Fixture { resource_manager, peer_id } = fixture().await?;
        let options = PeerMessagingBrokerOptions {
            peer_disconnect_timeout: Duration::from_millis(200),
        };
        let testee = PeerMessagingBroker::new(Arc::clone(&resource_manager), options.clone()).await;
        let remote_host = IpAddr::from_str("1.2.3.4")?;

        let result = testee.open(peer_id, remote_host, stream_header::ExtraHeaders::default()).await;
        assert!(result.is_ok());
        assert!(resource_manager.get::<PeerConnectionState>(peer_id).await?.is_some());
        assert!(resource_manager.get::<PeerDescriptor>(peer_id).await?.is_some());

        // ACT
        let _ = resource_manager.remove::<PeerDescriptor>(peer_id).await?;
        tokio::time::sleep(
            options.peer_disconnect_timeout * 6  // more than receive timeout + channel close timeout
        ).await;

        // ASSERT
        assert!(resource_manager.get::<PeerConnectionState>(peer_id).await?.is_none(), "Expected connection state to be remove since peer was removed!");
        Ok(())
    }

    #[tokio::test]
    async fn should_memorize_peer_connection_state_is_offline_when_peer_disconnects() -> anyhow::Result<()> {
        let Fixture { resource_manager, peer_id } = fixture().await?;
        let options = PeerMessagingBrokerOptions {
            peer_disconnect_timeout: Duration::from_millis(200),
        };
        let testee = PeerMessagingBroker::new(Arc::clone(&resource_manager), options.clone()).await;
        let remote_host = IpAddr::from_str("1.2.3.4")?;

        let result = testee.open(peer_id, remote_host, stream_header::ExtraHeaders::default()).await;
        assert!(result.is_ok());
        assert!(resource_manager.get::<PeerConnectionState>(peer_id).await?.is_some());

        // ACT
        testee.disconnect(peer_id).await?;
        tokio::time::sleep(
            options.peer_disconnect_timeout * 6  // more than receive timeout + channel close timeout
        ).await;

        // ASSERT
        let connection_state = resource_manager.get::<PeerConnectionState>(peer_id).await?;
        assert!(connection_state.is_some(), "Expected connection state to be present since peer was only disconnected and not removed!");
        assert_eq!(Some(PeerConnectionState::Offline), connection_state);
        
        Ok(())
    }

    #[tokio::test]
    async fn should_reject_second_connection_for_peer() -> anyhow::Result<()> {
        let Fixture { resource_manager, peer_id } = fixture().await?;

        let options = PeerMessagingBrokerOptions {
            peer_disconnect_timeout: Duration::from_millis(200),
        };
        let testee = PeerMessagingBroker::new(Arc::clone(&resource_manager), options.clone()).await;

        let remote_host = IpAddr::from_str("1.2.3.4")?;

        let result = testee.open(peer_id, remote_host, stream_header::ExtraHeaders::default()).await;
        assert!(result.is_ok());

        let result = testee.open(peer_id, remote_host, stream_header::ExtraHeaders::default()).await;
        assert_that!(
            result.unwrap_err(),
            matches_pattern!(OpenError::PeerAlreadyConnected { peer_id: eq(&peer_id) })
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
        resource_manager: ResourceManagerRef,
        peer_id: PeerId,
    }
    async fn fixture() -> anyhow::Result<Fixture> {
        let resource_manager = ResourceManager::new_in_memory();

        let peer_id = PeerId::random();
        let peer_descriptor = create_peer_descriptor(peer_id);
        resource_manager.insert(peer_id, peer_descriptor).await?;

        Ok(Fixture {
            resource_manager,
            peer_id,
        })
    }
}
