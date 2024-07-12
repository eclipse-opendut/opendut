use std::any::Any;
use std::fmt::Debug;
use std::ops::Not;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use opendut_carl_api::proto::services::peer_messaging_broker;
use opendut_carl_api::proto::services::peer_messaging_broker::downstream::Message;
use opendut_carl_api::proto::services::peer_messaging_broker::{ApplyPeerConfiguration, TracingContext};
use opendut_carl_api::proto::services::peer_messaging_broker::downstream::Message;
use opendut_restbus_simulation::arxml_parser::ArxmlParser;
use opendut_restbus_simulation::arxml_utils::get_timed_can_frames_from_bus;
use opendut_restbus_simulation::restbus_simulation::RestbusSimulation;
use opendut_restbus_simulation::restbus_structs::TimedCanFrame;
use opendut_types::cluster::{ClusterAssignment, PeerClusterAssignment};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use opendut_types::peer::PeerId;
use opendut_util::settings::LoadedConfig;
use opendut_util::telemetry;
use opendut_util::telemetry::logging::LoggingConfig;
use opendut_util::telemetry::opentelemetry_types::Opentelemetry;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tonic::Code;
use tracing::{debug, error, info, trace, warn, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::common::{carl, settings};
use crate::service::can_manager::{CanManager, CanManagerRef};
use crate::service::network_interface::manager::{NetworkInterfaceManager, NetworkInterfaceManagerRef};
use crate::service::peer_configuration::{ApplyPeerConfigurationParams, ClusterMetricsOptions, NetworkInterfaceManagement};
use crate::service::test_execution::executor_manager::{ExecutorManager, ExecutorManagerRef};
use crate::service::vpn;

const BANNER: &str = r"
                         _____     _______
                        |  __ \   |__   __|
   ___  _ __   ___ _ __ | |  | |_   _| |
  / _ \| '_ \ / _ \ '_ \| |  | | | | | |
 | (_) | |_) |  __/ | | | |__| | |_| | |
  \___/| .__/ \___|_| |_|_____/ \__,_|_|
       | |  ______ _____   _____          _____
       |_| |  ____|  __ \ / ____|   /\   |  __ \
           | |__  | |  | | |  __   /  \  | |__) |
           |  __| | |  | | | |_ | / /\ \ |  _  /
           | |____| |__| | |__| |/ ____ \| | \ \
           |______|_____/ \_____/_/    \_\_|  \_\";

pub async fn launch(id_override: Option<PeerId>) -> anyhow::Result<()> {
    println!("{}", crate::app_info::formatted_with_banner(BANNER));

    let settings_override = config::Config::builder()
        .set_override_option(settings::key::peer::id, id_override.map(|id| id.to_string()))?
        .build()?;

    create_with_telemetry(settings_override).await
}

pub async fn create_with_telemetry(settings_override: config::Config) -> anyhow::Result<()> {
    let settings = settings::load_with_overrides(settings_override)?;

    let self_id = settings.config.get::<PeerId>(settings::key::peer::id)
        .context("Failed to read ID from configuration.\n\nRun `edgar setup` before launching the service.")?;

    let service_instance_id = self_id.to_string();

    let logging_config = LoggingConfig::load(&settings.config)?;
    let opentelemetry = Opentelemetry::load(&settings.config, service_instance_id).await?;
    
    let mut shutdown = telemetry::initialize_with_config(logging_config, opentelemetry.clone()).await?;

    if let (Opentelemetry::Enabled { cpu_collection_interval_ms, .. },
        Some(meter_providers, ..)) = (opentelemetry, &shutdown.meter_providers) {
        telemetry::metrics::initialize_metrics_collection(cpu_collection_interval_ms, meter_providers);
    }

    let (tx_peer_configuration, rx_peer_configuration) = mpsc::channel(100);
    crate::service::peer_configuration::spawn_peer_configurations_handler(rx_peer_configuration).await?;

    run_stream_receiver(self_id, settings, tx_peer_configuration).await?;

    shutdown.shutdown();

    Ok(())
}

pub async fn run_stream_receiver(
    self_id: PeerId,
    settings: LoadedConfig,
    tx_peer_configuration: mpsc::Sender<ApplyPeerConfigurationParams>,
) -> anyhow::Result<()> {

    info!("Started with ID <{self_id}> and configuration: {settings:?}");

    let handle_stream_info = {
        let executor_manager: ExecutorManagerRef = ExecutorManager::create();

        let network_interface_management = {
            let network_interface_management_enabled = settings.config.get::<bool>("network.interface.management.enabled")?;
            if network_interface_management_enabled {
                let network_interface_manager: NetworkInterfaceManagerRef = NetworkInterfaceManager::create()?;
                let can_manager: CanManagerRef = CanManager::create(Arc::clone(&network_interface_manager));

                NetworkInterfaceManagement::Enabled { network_interface_manager, can_manager }
            } else {
                NetworkInterfaceManagement::Disabled
            }
        };

        let ping_interval = Duration::from_millis(settings.config.get::<u64>("opentelemetry.metrics.cluster.ping.interval.ms")?);
        let target_bandwidth_kbit_per_second = settings.config.get::<u64>("opentelemetry.metrics.cluster.target.bandwidth.kilobit.per.second")?;
        let rperf_backoff_max_elapsed_time = Duration::from_millis(settings.config.get::<u64>("opentelemetry.metrics.cluster.rperf.backoff.max.elapsed.time.ms")?);

        HandleStreamInfo {
            self_id,
            network_interface_management,
            executor_manager,
            cluster_metrics_options: ClusterMetricsOptions {
                ping_interval,
                target_bandwidth_kbit_per_second,
                rperf_backoff_max_elapsed_time,
            },
        }
    };
    
    let restbus_simulation: RestbusSimulation = RestbusSimulation {};
    
    let restbus_simulation_enabled = settings.config.get::<bool>("restbus-simulation.enabled")?;

    if restbus_simulation_enabled {
        let arxml_file_path = settings.config.get::<String>("restbus-simulation.arxml.path")?;
        let arxml_serialization = settings.config.get::<bool>("restbus-simulation.arxml.serialization")?;
        let target_cluster = settings.config.get::<String>("restbus-simulation.simulation.target.cluster")?;
        let ifname = settings.config.get::<String>("restbus-simulation.simulation.interface")?;
        
        let arxml_parser: ArxmlParser = ArxmlParser {};

        if let Some(can_clusters) = arxml_parser.parse_file(&arxml_file_path, arxml_serialization) {
            // Play all CAN frames from the target bus periodcically (only periodic frames are sent periodically) to the bus to which ifname is connected to.
            let timed_can_frames: Vec<TimedCanFrame> = get_timed_can_frames_from_bus(&can_clusters, target_cluster);

            match restbus_simulation.play_all(&timed_can_frames, &ifname) {
                Ok(_val) => info!("Successfully established restbus simulation"),
                Err(error) => info!("Could not establish restbus simulation because: {}", error)
            }
        }
    }

    let remote_address = vpn::retrieve_remote_host(&settings).await?;
    
    let timeout_duration = Duration::from_millis(settings.config.get::<u64>("carl.disconnect.timeout.ms")?);

    let mut carl = carl::connect(&settings.config).await?;

    let (mut rx_inbound, tx_outbound) = carl::open_stream(self_id, &remote_address, &mut carl).await?;

    loop {
        let received = tokio::time::timeout(timeout_duration, rx_inbound.message()).await;

        match received {
            Ok(received) => match received {
                Ok(Some(message)) => {
                    handle_stream_message(
                        message,
                        &handle_stream_info,
                        &tx_outbound,
                        &tx_peer_configuration,
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


struct HandleStreamInfo {
    pub self_id: PeerId,
    pub network_interface_management: NetworkInterfaceManagement,
    pub executor_manager: ExecutorManagerRef,
    pub cluster_metrics_options: ClusterMetricsOptions,
}

async fn handle_stream_message(
    message: peer_messaging_broker::Downstream,
    handle_stream_info: &HandleStreamInfo,
    tx_outbound: &mpsc::Sender<peer_messaging_broker::Upstream>,
    peer_configuration_sender: &mpsc::Sender<ApplyPeerConfigurationParams>,
) -> anyhow::Result<()> {

    if let peer_messaging_broker::Downstream { message: Some(message), context } = message {
        if matches!(message, Message::Pong(_)).not() {
            trace!("Received message: {:?}", message);
        }

        match message {
            Message::Pong(_) => {
                sleep(Duration::from_secs(5)).await;
                let message = peer_messaging_broker::Upstream {
                    message: Some(peer_messaging_broker::upstream::Message::Ping(peer_messaging_broker::Ping {})),
                    context: None
                };
                let _ignore_error =
                    tx_outbound.send(message).await
                        .inspect_err(|cause| debug!("Failed to send ping to CARL: {cause}"));
            }
            Message::ApplyPeerConfiguration(message) => apply_peer_configuration_raw(message, context, handle_stream_info, peer_configuration_sender).await?,
        }
    } else {
        ignore(message)
    }

    Ok(())
}

async fn apply_peer_configuration_raw(
    message: ApplyPeerConfiguration,
    context: Option<TracingContext>,
    handle_stream_info: &HandleStreamInfo,
    peer_configuration_sender: &mpsc::Sender<ApplyPeerConfigurationParams>,
) -> anyhow::Result<()> {

    match message.clone() {
        ApplyPeerConfiguration {
            old_configuration: Some(old_peer_configuration),
            configuration: Some(peer_configuration),
        } => {

            let span = Span::current();
            set_parent_context(&span, context);
            let _span = span.enter();

            match OldPeerConfiguration::try_from(old_peer_configuration) {
                Ok(old_peer_configuration) => {
                    match PeerConfiguration::try_from(peer_configuration) {
                        Ok(peer_configuration) => {
                            info!("Received OldPeerConfiguration: {old_peer_configuration:?}");
                            info!("Received PeerConfiguration: {peer_configuration:?}");
                            
                            let apply_config_params = ApplyPeerConfigurationParams {
                                self_id: handle_stream_info.self_id,
                                peer_configuration,
                                old_peer_configuration,
                                network_interface_management: handle_stream_info.network_interface_management.clone(),
                                executor_manager: Arc::clone(&handle_stream_info.executor_manager),
                                cluster_metrics_options: handle_stream_info.cluster_metrics_options.clone(),
                            };
                            peer_configuration_sender.send(apply_config_params).await?
                        }
                        Err(error) => error!("Illegal PeerConfiguration: {error}"),
                    }
                }
                Err(error) => error!("Illegal OldPeerConfiguration: {error}"),
            };
        }
        _ => ignore(message),
    }
    Ok(())
}

fn set_parent_context(span: &Span, context: Option<TracingContext>) {
    if let Some(context) = context {
        let propagator = TraceContextPropagator::new();
        let parent_context = propagator.extract(&context.values);
        span.set_parent(parent_context);
    }
}

fn ignore(message: impl Any + Debug) {
    warn!("Ignoring illegal message: {message:?}");
}
