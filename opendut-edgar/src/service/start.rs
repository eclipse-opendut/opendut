use std::any::Any;
use std::fmt::Debug;
use std::ops::Not;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use config::Config;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use opendut_carl_api::proto::services::peer_messaging_broker;
use opendut_carl_api::proto::services::peer_messaging_broker::AssignCluster;
use opendut_carl_api::proto::services::peer_messaging_broker::downstream::Message;
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::PeerId;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_util::logging;

use crate::common::{carl, settings};
use crate::service::{cluster_assignment, vpn};
use crate::service::can_manager::{CanManager, CanManagerRef};
use crate::service::network_interface::manager::{NetworkInterfaceManager, NetworkInterfaceManagerRef};

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

    logging::initialize()?;

    let settings_override = Config::builder()
        .set_override_option(settings::key::peer::id, id_override.map(|id| id.to_string()))?
        .build()?;

    create(settings_override).await
}

pub async fn create(settings_override: Config) -> anyhow::Result<()> {

    let settings = settings::load_with_overrides(settings_override)?;
    let id = settings.config.get::<PeerId>(settings::key::peer::id)
        .context("Failed to read ID from configuration.\n\nRun `edgar setup` before launching the service.")?;

    log::info!("Started with ID <{id}> and configuration: {settings:?}");

    let network_interface_manager: NetworkInterfaceManagerRef = Arc::new(NetworkInterfaceManager::create()?);
    let can_manager: CanManagerRef = Arc::new(CanManager::create(Arc::clone(&network_interface_manager)));

    let network_interface_management_enabled = settings.config.get::<bool>("network.interface.management.enabled")?;

    let bridge_name = crate::common::default_bridge_name();

    let remote_address = vpn::retrieve_remote_host(&settings).await?;

    let timeout_duration = Duration::from_millis(settings.config.get::<u64>("carl.disconnect.timeout.ms")?);

    log::debug!("Connecting to CARL...");
    let mut carl = carl::connect(&settings.config).await?;
    log::debug!("Connected to CARL.");

    log::info!("Connecting to peer-messaging-broker...");

    let (mut rx_inbound, tx_outbound) = carl.broker.open_stream(id, remote_address).await?;

    let message = peer_messaging_broker::Upstream {
        message: Some(peer_messaging_broker::upstream::Message::Ping(peer_messaging_broker::Ping {}))
    };

    tx_outbound.send(message).await
        .map_err(|cause| opendut_carl_api::carl::broker::error::OpenStream { message: format!("Error while sending initial ping: {cause}") })?;


    loop {
        let received = tokio::time::timeout(timeout_duration, rx_inbound.message()).await;

        match received {
            Ok(received) => match received {
                Ok(Some(message)) => {
                    handle_stream_message(
                        message,
                        id,
                        network_interface_management_enabled,
                        &bridge_name,
                        Arc::clone(&network_interface_manager),
                        &tx_outbound,
                    ).await?
                }
                Err(status) => {
                    log::warn!("CARL sent a gRPC error status: {status}");
                    //TODO exit?
                }
                Ok(None) => {
                    log::info!("CARL disconnected!");
                    break;
                }
            }
            Err(_) => {
                log::error!("No message from CARL within {} ms.", timeout_duration.as_millis());
                break;
            }
        }
    }

    Ok(())
}


async fn handle_stream_message(
    message: peer_messaging_broker::Downstream,
    self_id: PeerId,
    network_interface_management_enabled: bool,
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
    tx_outbound: &Sender<peer_messaging_broker::Upstream>,
) -> anyhow::Result<()> {
    fn ignore(message: impl Any + Debug) {
        log::warn!("Ignoring illegal message: {message:?}");
    }

    if let peer_messaging_broker::Downstream { message: Some(message) } = message {
        if matches!(message, Message::Pong(_)).not() {
            log::trace!("Received message: {:?}", message);
        }

        match message {
            Message::Pong(_) => {
                sleep(Duration::from_secs(5)).await;
                let message = peer_messaging_broker::Upstream {
                    message: Some(peer_messaging_broker::upstream::Message::Ping(peer_messaging_broker::Ping {}))
                };
                let _ignore_error =
                    tx_outbound.send(message).await
                        .inspect_err(|cause| log::debug!("Failed to send ping to CARL: {cause}"));
            }
            Message::AssignCluster(message) => match message {
                AssignCluster { assignment: Some(cluster_assignment) } => {
                    let cluster_assignment = ClusterAssignment::try_from(cluster_assignment)?;
                    log::trace!("Received ClusterAssignment: {cluster_assignment:?}");
                    log::info!("Was assigned to cluster <{}>", cluster_assignment.id);

                    if network_interface_management_enabled {
                        cluster_assignment::network_interfaces_setup(
                            cluster_assignment,
                            self_id,
                            bridge_name,
                            Arc::clone(&network_interface_manager),
                                Arc::clone(&can_manager)
                        ).await
                        .inspect_err(|error| {
                            log::error!("Failed to configure network interfaces: {error}")
                        })?;
                    } else {
                        log::debug!("Skipping changes to network interfaces after receiving ClusterAssignment, as this is disabled via configuration.");
                    }
                }
                _ => ignore(message),
            }
            Message::ApplyVpnConfig(_) => {}
        }
    } else {
        ignore(message)
    }

    Ok(())
}
