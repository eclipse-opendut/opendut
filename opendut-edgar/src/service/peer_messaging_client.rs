use std::net::IpAddr;
use std::ops::Not;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tonic::Code;
use tracing::{debug, error, info, trace, warn, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use opendut_carl_api::carl::{broker, GrpcUpstream};
use opendut_carl_api::carl::broker::Upstream;
use opendut_carl_api::carl::CarlClient;
use opendut_model::format::DebugJsonDisplay;
use opendut_model::peer::configuration::EdgePeerConfigurationState;
use opendut_model::peer::PeerId;
use opendut_util::settings::LoadedConfig;
use crate::common::carl;
use crate::service::can::can_manager::CanManager;
use crate::service::network_interface::manager::{NetworkInterfaceManager, NetworkInterfaceManagerRef};
use crate::service::network_metrics::manager::{NetworkMetricsManager, NetworkMetricsManagerRef};
use crate::service::peer_configuration::{ApplyPeerConfigurationParams, NetworkInterfaceManagement};
use crate::service::test_execution::executor_manager::{ExecutorManager, ExecutorManagerRef};


pub struct PeerMessagingClient {
    self_id: PeerId,
    network_interface_management: NetworkInterfaceManagement,
    executor_manager: ExecutorManagerRef,
    metrics_manager: NetworkMetricsManagerRef,
    carl_disconnect_timeout: Duration,
    tx_peer_configuration: mpsc::Sender<ApplyPeerConfigurationParams>,
}


impl PeerMessagingClient {
    pub async fn create(
       self_id: PeerId,
       settings: &LoadedConfig,
       tx_peer_configuration: mpsc::Sender<ApplyPeerConfigurationParams>,
    ) -> anyhow::Result<Self> {
        info!("Started with ID <{self_id}> and configuration: {settings:?}");

        let carl_disconnect_timeout = Duration::from_millis(settings.get::<u64>("carl.disconnect.timeout.ms")?);

        let executor_manager: ExecutorManagerRef = ExecutorManager::create();

        let network_interface_management = {
            let network_interface_management_enabled = settings.get::<bool>("network.interface.management.enabled")?;
            if network_interface_management_enabled {
                let network_interface_manager: NetworkInterfaceManagerRef = NetworkInterfaceManager::create()?;
                let can_manager = CanManager::create();

                NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager }
            } else {
                NetworkInterfaceManagement::Disabled
            }
        };

        let metrics_manager: NetworkMetricsManagerRef = NetworkMetricsManager::load(settings)?;


        Ok(PeerMessagingClient {
            self_id,
            network_interface_management,
            executor_manager,
            metrics_manager,
            carl_disconnect_timeout,
            tx_peer_configuration,
        })
    }

    pub async fn process_messages_loop(
        &self,
        carl: &mut CarlClient,
        rx_peer_configuration_state: Arc<Mutex<Receiver<EdgePeerConfigurationState>>>,
        remote_address: &IpAddr,
        on_connect_success: &impl Fn(),
        cancel_token: &CancellationToken,
    ) -> anyhow::Result<()> {

        let (mut rx_inbound, tx_outbound) = carl::open_stream(self.self_id, remote_address, carl).await?;

        self.spawn_peer_configuration_state_sender(rx_peer_configuration_state, tx_outbound.clone()).await;

        on_connect_success();

        loop {
            tokio::select! {
                received = tokio::time::timeout(self.carl_disconnect_timeout, rx_inbound.receive()) => {
                    match received {
                        Ok(received) => match received {
                            Ok(Some(message)) => {
                                self.handle_stream_message(
                                    message,
                                    &tx_outbound,
                                    &self.tx_peer_configuration,
                                    cancel_token,
                                ).await?
                            }
                            Err(status) => {
                                warn!("CARL sent a gRPC error status: {status}");

                                match status.code() {
                                    Code::Ok | Code::AlreadyExists => continue, //ignore

                                    Code::DeadlineExceeded | Code::Unavailable => { //ignore, but delay reading the stream again, as this may result in rapid triggering of errors otherwise
                                        tokio::time::sleep(Duration::from_secs(1)).await;
                                        continue
                                    }

                                    Code::Unknown => {
                                        debug!("Triggering reconnect to CARL after receiving gRPC error status.");
                                        break
                                    }

                                    Code::Aborted
                                    | Code::Cancelled
                                    | Code::DataLoss
                                    | Code::FailedPrecondition
                                    | Code::Internal
                                    | Code::InvalidArgument
                                    | Code::NotFound
                                    | Code::OutOfRange
                                    | Code::PermissionDenied
                                    | Code::ResourceExhausted
                                    | Code::Unimplemented
                                    | Code::Unauthenticated
                                    => panic!("Received potentially bad gRPC error: {status}"), //In production, SystemD will restart EDGAR with a delay. A crash is mainly more visible.
                                }
                            }
                            Ok(None) => {
                                info!("CARL disconnected!");
                                break;
                            }
                        }
                        Err(_) => {
                            error!("No message from CARL within {:?}.", self.carl_disconnect_timeout);
                            break;
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    debug!("PeerMessagingClient message processing is being cancelled.");
                    break;
                }
            }
        }

        Ok(())
    }

    pub(super) async fn destroy(self) {
        // Shutdown processes of the CAN manager if enabled
        match self.network_interface_management.clone() {
            NetworkInterfaceManagement::Enabled { can_manager, .. } => {
                let can_manager = can_manager.lock().await;
                can_manager.shutdown().await;
            }
            NetworkInterfaceManagement::Disabled => {}
        }
    }


    async fn spawn_peer_configuration_state_sender(
        &self,
        rx_peer_configuration_state: Arc<Mutex<Receiver<EdgePeerConfigurationState>>>,
        tx_outbound: Upstream,
    ) {
        tokio::spawn(async move {
            loop {
                let message = rx_peer_configuration_state.lock().await
                    .recv().await;

                match message {
                    None => {
                        info!("Peer configuration state channel closed.");
                        break  // exit the loop and end the EdgePeerConfigurationState sender task
                    }
                    Some(message) => {
                        let _send_result = tx_outbound.send(message.clone()).await
                            .inspect_err(|error| {
                                error!("Failed to send PeerConfigurationState {message:?} to CARL. Encountered error was: {error}");
                            });
                    }
                }
            }
        });
    }


    async fn handle_stream_message(
        &self,
        message: broker::DownstreamMessage,
        tx_outbound: &GrpcUpstream,
        peer_configuration_sender: &mpsc::Sender<ApplyPeerConfigurationParams>,
        cancel_token: &CancellationToken,
    ) -> anyhow::Result<()> {
        let broker::DownstreamMessage { payload: message, context } = message;

        if !matches!(message, broker::DownstreamMessagePayload::Pong) {
            trace!("Received message: {}", message.to_debug_json());
        }

        match message {
            broker::DownstreamMessagePayload::Pong => {
                tokio::select! {
                    _ = sleep(Duration::from_secs(5)) => {
                        let message = broker::UpstreamMessage {
                            payload: broker::UpstreamMessagePayload::Ping,
                            context: None,
                        };
                        let _ignore_error =
                            tx_outbound.send(message).await
                                .inspect_err(|cause| debug!("Failed to send ping to CARL: {cause:?}"));
                    }
                    _ = cancel_token.cancelled() => {
                        debug!("Responding with Pong message cancelled.");
                    }
                }
            }
            broker::DownstreamMessagePayload::ApplyPeerConfiguration(message) => self.apply_peer_configuration_raw(message, context, peer_configuration_sender).await?,
            broker::DownstreamMessagePayload::DisconnectNotice => {
                return Err(anyhow!("CARL sent a disconnect notice. Shutting down now."))
            }
        }

        Ok(())
    }


    async fn apply_peer_configuration_raw(
        &self,
        message: Box<broker::ApplyPeerConfiguration>,
        context: Option<broker::TracingContext>,
        peer_configuration_sender: &mpsc::Sender<ApplyPeerConfigurationParams>,
    ) -> anyhow::Result<()> {

        let span = Span::current();
        set_parent_context(&span, context)?;
        let _span = span.enter();

        let broker::ApplyPeerConfiguration { configuration } = *message;

        let apply_config_params = ApplyPeerConfigurationParams {
            peer_configuration: configuration,
            network_interface_management: self.network_interface_management.clone(),
            executor_manager: Arc::clone(&self.executor_manager),
            metrics_manager: Arc::clone(&self.metrics_manager),
        };
        peer_configuration_sender.send(apply_config_params).await?;

        Ok(())
    }
}


fn set_parent_context(span: &Span, context: Option<broker::TracingContext>) -> anyhow::Result<()> {
    if let Some(context) = context {
        let propagator = TraceContextPropagator::new();
        let parent_context = propagator.extract(&context.values);
        if span.is_disabled().not() {
            span.set_parent(parent_context)?;
        }
    }
    Ok(())
}
