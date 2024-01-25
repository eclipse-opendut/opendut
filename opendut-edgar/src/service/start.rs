use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use config::Config;
use tokio::time::sleep;

use opendut_carl_api::proto::services::peer_messaging_broker;
use opendut_carl_api::proto::services::peer_messaging_broker::{AssignCluster, Downstream};
use opendut_carl_api::proto::services::peer_messaging_broker::downstream::Message;
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::PeerId;
use opendut_types::util::net::NetworkInterfaceName;
use opendut_util::logging;

use crate::common::{carl, settings};
use crate::service::network_interface::manager::{NetworkInterfaceManager, NetworkInterfaceManagerRef};
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

    let network_interface_management_enabled = settings.config.get::<bool>("network.interface.management.enabled")?;

    let bridge_name = crate::common::default_bridge_name();

    if network_interface_management_enabled {
        create_bridge(&bridge_name, Arc::clone(&network_interface_manager)).await?;
    }

    let remote_address = vpn::retrieve_remote_host(&settings).await?;

    log::debug!("Connecting to CARL...");
    let mut carl = carl::connect(&settings.config).await?;
    log::debug!("Connected to CARL.");

    log::info!("Connecting to peer-messaging-broker...");

    let (mut inbound, tx) = carl.broker.open_stream(id, remote_address).await?;

    let message = peer_messaging_broker::Upstream {
        message: Some(peer_messaging_broker::upstream::Message::Ping(peer_messaging_broker::Ping {}))
    };

    tx.send(message).await
        .map_err(|cause| opendut_carl_api::carl::broker::error::OpenStream { message: format!("Error while sending initial ping: {cause}") })?;

    while let Some(message) = inbound.message().await? {

        fn ignore(message: impl Any + Debug) {
            log::warn!("Ignoring illegal message: {message:?}");
        }

        if let Downstream { message: Some(message) } = message {
            log::trace!("Received message: {:?}", message);

            match message {
                Message::Pong(_) => {
                    sleep(Duration::from_secs(5)).await;
                    let message = peer_messaging_broker::Upstream {
                        message: Some(peer_messaging_broker::upstream::Message::Ping(peer_messaging_broker::Ping {}))
                    };
                    tx.send(message).await?;
                }
                Message::AssignCluster(message) => match message {
                    AssignCluster { assignment: Some(cluster_assignment) } => {
                        let cluster_assignment = ClusterAssignment::try_from(cluster_assignment)?;
                        log::trace!("Received ClusterAssignment: {cluster_assignment:?}");
                        log::info!("Was assigned to cluster <{}>", cluster_assignment.id);

                        crate::service::cluster_assignment::handle(
                            cluster_assignment,
                            id,
                            &bridge_name,
                            Arc::clone(&network_interface_manager),
                        ).await?;
                    }
                    _ => ignore(message),
                }
                Message::ApplyVpnConfig(_) => {}
            }
        } else {
            ignore(message)
        }
    }

    Ok(())
}

async fn create_bridge(bridge_name: &NetworkInterfaceName, network_interface_manager: NetworkInterfaceManagerRef) -> anyhow::Result<()> {

    if network_interface_manager.find_interface(bridge_name).await?.is_none() {
        log::debug!("Creating bridge '{bridge_name}'.");
        crate::service::network_interface::bridge::create(
            bridge_name,
            network_interface_manager,
        ).await?;
    } else {
        log::debug!("Not creating bridge '{bridge_name}', because it already exists.");
    }

    Ok(())
}
