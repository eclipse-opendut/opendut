use crate::app_info;
use crate::common::{carl, settings};
use anyhow::Context;
use opendut_model::peer::configuration::EdgePeerConfigurationState;
use opendut_model::peer::PeerId;
use opendut_telemetry::logging::LoggingConfig;
use opendut_telemetry::opentelemetry_types;
use opendut_telemetry::opentelemetry_types::Opentelemetry;
use tokio::sync::mpsc;
use tracing::info;
use crate::service::peer_messaging_client::PeerMessagingClient;
use crate::service::vpn::VpnProcess;


pub async fn launch(id_override: Option<PeerId>) -> anyhow::Result<()> {
    crate::common::banner::print();

    let settings_override = config::Config::builder()
        .set_override_option(settings::key::peer::id, id_override.map(|id| id.to_string()))?
        .build()?;

    create_with_telemetry(settings_override).await
}

pub async fn create_with_telemetry(settings_override: config::Config) -> anyhow::Result<()> {
    let settings = settings::load_with_overrides(settings_override)?;

    let self_id = settings.get::<PeerId>(settings::key::peer::id)
        .context("Failed to read ID from configuration.\n\nRun `edgar setup` before launching the service.")?;

    let mut metrics_shutdown_handle = {
        let logging_config = LoggingConfig::load(&settings)?;
        let service_metadata = opentelemetry_types::ServiceMetadata {
            instance_id: format!("edgar-{self_id}"),
            version: app_info::PKG_VERSION.to_owned(),
        };
        let opentelemetry = Opentelemetry::load(&settings, service_metadata).await?;

        opendut_telemetry::initialize_with_config(logging_config, opentelemetry).await?
    };

    log_edgar_metadata(self_id)?;


    let vpn = VpnProcess::spawn_from_config(&settings).await?;


    let (tx_peer_configuration, rx_peer_configuration) = mpsc::channel(100);
    let (tx_peer_configuration_state, rx_peer_configuration_state) = mpsc::channel::<EdgePeerConfigurationState>(100);
    crate::service::peer_configuration::spawn_peer_configurations_handler(rx_peer_configuration, tx_peer_configuration_state).await?;

    let mut carl = carl::connect(&settings).await?;
    carl::log_version_compatibility(&mut carl).await?;

    let remote_address = vpn.retrieve_remote_host(&settings).await?;

    let mut peer_messaging_client = PeerMessagingClient::create(self_id, carl, settings, tx_peer_configuration).await?;
    peer_messaging_client.process_messages_loop(rx_peer_configuration_state, remote_address).await?;

    {
        info!("EDGAR is terminating...");

        vpn.terminate().await?;

        metrics_shutdown_handle.shutdown();
    }
    Ok(())
}

fn log_edgar_metadata(self_id: PeerId) -> anyhow::Result<()> {
    let user = nix::unistd::User::from_uid(
        nix::unistd::getuid()
    )?;

    let user = match user {
        Some(user) => format!("system user '{}'", user.name),
        None => String::from("an unknown system user"),
    };

    info!("Running with PeerId <{self_id}> under {user}.");
    Ok(())
}
