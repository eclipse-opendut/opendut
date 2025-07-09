use crate::app_info;
use crate::common::{carl, settings};
use anyhow::Context;
use opendut_types::peer::configuration::PeerConfigurationState;
use opendut_types::peer::PeerId;
use opendut_util::telemetry;
use opendut_util::telemetry::logging::LoggingConfig;
use opendut_util::telemetry::opentelemetry_types;
use opendut_util::telemetry::opentelemetry_types::Opentelemetry;
use tokio::sync::mpsc;
use crate::service::peer_messaging_client::PeerMessagingClient;

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
    println!("{BANNER}\n{version_info}", version_info=crate::FORMATTED_VERSION);

    let settings_override = config::Config::builder()
        .set_override_option(settings::key::peer::id, id_override.map(|id| id.to_string()))?
        .build()?;

    create_with_telemetry(settings_override).await
}

pub async fn create_with_telemetry(settings_override: config::Config) -> anyhow::Result<()> {
    let settings = settings::load_with_overrides(settings_override)?;

    let self_id = settings.config.get::<PeerId>(settings::key::peer::id)
        .context("Failed to read ID from configuration.\n\nRun `edgar setup` before launching the service.")?;

    let mut metrics_shutdown_handle = {
        let logging_config = LoggingConfig::load(&settings.config)?;
        let service_metadata = opentelemetry_types::ServiceMetadata {
            instance_id: format!("edgar-{self_id}"),
            version: app_info::PKG_VERSION.to_owned(),
        };
        let opentelemetry = Opentelemetry::load(&settings.config, service_metadata).await?;

        telemetry::initialize_with_config(logging_config, opentelemetry).await?
    };

    let (tx_peer_configuration, rx_peer_configuration) = mpsc::channel(100);
    let (tx_peer_configuration_state, rx_peer_configuration_state) = mpsc::channel::<PeerConfigurationState>(100);
    crate::service::peer_configuration::spawn_peer_configurations_handler(rx_peer_configuration, tx_peer_configuration_state).await?;

    let mut carl = carl::connect(&settings.config).await?;
    carl::log_version_compatibility(&mut carl).await?;
    let mut peer_messaging_client = PeerMessagingClient::create(self_id, carl, settings, tx_peer_configuration).await?;
    peer_messaging_client.process_messages_loop(rx_peer_configuration_state).await?;

    metrics_shutdown_handle.shutdown();

    Ok(())
}

