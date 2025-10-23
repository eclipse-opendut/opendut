use std::ops::Not;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;
use tonic::Code;
use tracing::{debug, error, info, trace, warn, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use opendut_carl_api::carl::{broker, GrpcUpstream};
use opendut_carl_api::carl::broker::Upstream;
use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::configuration::EdgePeerConfigurationState;
use opendut_model::peer::PeerId;
use opendut_util::settings::LoadedConfig;
use crate::common::carl;
use crate::service::can::can_manager::CanManager;
use crate::service::network_interface::manager::{NetworkInterfaceManager, NetworkInterfaceManagerRef};
use crate::service::network_metrics::manager::{NetworkMetricsManager, NetworkMetricsManagerRef};
use crate::service::peer_configuration::{ApplyPeerConfigurationParams, NetworkInterfaceManagement};
use crate::service::test_execution::executor_manager::{ExecutorManager, ExecutorManagerRef};
use crate::service::vpn;

pub struct PeerMessagingClient {
    carl: CarlClient,
    handle_stream_info: HandleStreamInfo,
    settings: LoadedConfig,
    tx_peer_configuration: mpsc::Sender<ApplyPeerConfigurationParams>,
}

pub struct HandleStreamInfo {
    pub self_id: PeerId,
    pub network_interface_management: NetworkInterfaceManagement,
    pub executor_manager: ExecutorManagerRef,
    pub metrics_manager: NetworkMetricsManagerRef,
}

impl PeerMessagingClient {
    pub async fn create(
       self_id: PeerId,
       carl: CarlClient,
       settings: LoadedConfig,
       tx_peer_configuration: mpsc::Sender<ApplyPeerConfigurationParams>,
    ) -> anyhow::Result<Self> {
        info!("Started with ID <{self_id}> and configuration: {settings:?}");

        let handle_stream_info = {
            let executor_manager: ExecutorManagerRef = ExecutorManager::create();

            let network_interface_management = {
                let network_interface_management_enabled = settings.config.get::<bool>("network.interface.management.enabled")?;
                if network_interface_management_enabled {
                    let network_interface_manager: NetworkInterfaceManagerRef = NetworkInterfaceManager::create()?;
                    let can_manager = CanManager::create(Arc::clone(&network_interface_manager));

                    NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager }
                } else {
                    NetworkInterfaceManagement::Disabled
                }
            };

            let metrics_manager: NetworkMetricsManagerRef = NetworkMetricsManager::load(&settings)?;


            HandleStreamInfo {
                self_id,
                network_interface_management,
                executor_manager,
                metrics_manager,
            }
        };

        Ok(PeerMessagingClient {
            carl,
            handle_stream_info,
            settings,
            tx_peer_configuration,
        })
    }
    
    async fn spawn_peer_configuration_state_sender(&self, mut rx_peer_configuration_state: Receiver<EdgePeerConfigurationState>, tx_outbound: Upstream) {
        tokio::spawn(async move {
            loop {
                let message = rx_peer_configuration_state.recv().await;
                match message {
                    None => {
                        info!("Peer configuration state channel closed");
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

    pub async fn process_messages_loop(&mut self, rx_peer_configuration_state: Receiver<EdgePeerConfigurationState>) -> anyhow::Result<()> {
        let remote_address = vpn::retrieve_remote_host(&self.settings).await?;

        let timeout_duration = Duration::from_millis(self.settings.config.get::<u64>("carl.disconnect.timeout.ms")?);

        let (mut rx_inbound, tx_outbound) = carl::open_stream(self.handle_stream_info.self_id, &remote_address, &mut self.carl).await?;

        self.spawn_peer_configuration_state_sender(rx_peer_configuration_state, tx_outbound.clone()).await;

        loop {
            let received = tokio::time::timeout(timeout_duration, rx_inbound.receive()).await;

            match received {
                Ok(received) => match received {
                    Ok(Some(message)) => {
                        self.handle_stream_message(
                            message,
                            &tx_outbound,
                            &self.tx_peer_configuration,
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
                            | Code::Unknown
                            => panic!("Received potentially bad gRPC error: {status}"), //In production, SystemD will restart EDGAR with a delay. A crash is mainly more visible.
                        }
                    }
                    Ok(None) => {
                        info!("CARL disconnected!");
                        break;
                    }
                }
                Err(_) => {
                    error!("No message from CARL within {} ms.", timeout_duration.as_millis());
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_stream_message(
        &self,
        message: broker::DownstreamMessage,
        tx_outbound: &GrpcUpstream,
        peer_configuration_sender: &mpsc::Sender<ApplyPeerConfigurationParams>,
    ) -> anyhow::Result<()> {
        let broker::DownstreamMessage { payload: message, context } = message;

        if !matches!(message, broker::DownstreamMessagePayload::Pong) {
            trace!("Received message: {message:?}");
        }

        match message {
            broker::DownstreamMessagePayload::Pong => {
                sleep(Duration::from_secs(5)).await;
                let message = broker::UpstreamMessage {
                    payload: broker::UpstreamMessagePayload::Ping,
                    context: None,
                };
                let _ignore_error =
                    tx_outbound.send(message).await
                        .inspect_err(|cause| debug!("Failed to send ping to CARL: {cause}"));
            }
            broker::DownstreamMessagePayload::ApplyPeerConfiguration(message) => apply_peer_configuration_raw(message, context, &self.handle_stream_info, peer_configuration_sender).await?,
            broker::DownstreamMessagePayload::DisconnectNotice => {
                return Err(anyhow!("CARL sent a disconnect notice. Shutting down now."))
            }
        }

        Ok(())
    }
}

async fn apply_peer_configuration_raw(
    message: Box<broker::ApplyPeerConfiguration>,
    context: Option<broker::TracingContext>,
    handle_stream_info: &HandleStreamInfo,
    peer_configuration_sender: &mpsc::Sender<ApplyPeerConfigurationParams>,
) -> anyhow::Result<()> {

    let span = Span::current();
    set_parent_context(&span, context)?;
    let _span = span.enter();

    let broker::ApplyPeerConfiguration { old_configuration, configuration } = *message;

    info!("Received OldPeerConfiguration: {old_configuration:?}");
    info!("Received PeerConfiguration: {configuration:?}");

    let apply_config_params = ApplyPeerConfigurationParams {
        self_id: handle_stream_info.self_id,
        peer_configuration: configuration,
        old_peer_configuration: old_configuration,
        network_interface_management: handle_stream_info.network_interface_management.clone(),
        executor_manager: Arc::clone(&handle_stream_info.executor_manager),
        metrics_manager: Arc::clone(&handle_stream_info.metrics_manager),
    };
    peer_configuration_sender.send(apply_config_params).await?;

    Ok(())
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
