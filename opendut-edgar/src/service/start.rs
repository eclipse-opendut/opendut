use crate::app_info;
use crate::common::settings;
use crate::service::peer_messaging_client::PeerMessagingClient;
use crate::service::vpn::VpnProcess;
use anyhow::Context;
use opendut_model::peer::configuration::EdgePeerConfigurationState;
use opendut_model::peer::PeerId;
use opendut_telemetry::logging::LoggingConfig;
use opendut_telemetry::opentelemetry_types;
use opendut_telemetry::opentelemetry_types::Opentelemetry;
use opendut_util::settings::LoadedConfig;
use std::net::IpAddr;
use std::ops::Not;
use std::sync::Arc;
use std::time::Duration;
use backon::{BackoffBuilder, ExponentialBuilder};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};
use opendut_carl_api::carl::CarlClient;
use opendut_util::pem;
use opendut_util::pem::{ClientAuth, Pem, PemFromConfig};
use crate::common::carl::log_version_compatibility;


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
    let remote_address = vpn.retrieve_remote_host(&settings).await?;


    let (tx_peer_configuration, rx_peer_configuration) = mpsc::channel(100);
    let (tx_peer_configuration_state, rx_peer_configuration_state) = mpsc::channel::<EdgePeerConfigurationState>(100);

    let connect_cancel = CancellationToken::new();
    crate::service::peer_configuration::spawn_peer_configurations_handler(
        rx_peer_configuration,
        tx_peer_configuration_state,
        connect_cancel.clone(),
    ).await?;


    let peer_messaging_client = PeerMessagingClient::create(self_id, &settings, tx_peer_configuration).await?;

    let rx_peer_configuration_state = Arc::new(Mutex::new(rx_peer_configuration_state));

    connect_and_start(
        &ConnectAndStart::Service {
            peer_messaging_client: &peer_messaging_client,
            rx_peer_configuration_state,
            remote_address,
        },
        &settings,
        connect_cancel.clone(),
    ).await?;

    {
        info!("EDGAR is terminating...");

        peer_messaging_client.destroy().await;

        connect_cancel.cancel();

        vpn.terminate().await?;

        metrics_shutdown_handle.shutdown();
    }
    Ok(())
}

pub async fn connect_and_start(config: &ConnectAndStart<'_>, settings: &LoadedConfig, connect_cancel: CancellationToken) -> anyhow::Result<()> {

    let ConnectOptions { host, port, ca_certs, client_auth, domain_name_override, retries, interval } =
        ConnectOptions::load_from_config(settings)?;

    let initial_backoff = ExponentialBuilder::new()
        .with_max_times(retries)
        .with_max_delay(interval);

    let backoff = Arc::new(Mutex::new(initial_backoff.build()));

    let on_connect_success = async || { //reset the number of retries after a connection is established
        let mut backoff = backoff.lock().await;

        *backoff = initial_backoff.build();
    };

    opendut_util::crypto::install_default_provider();

    loop {
        let result: anyhow::Result<()> = async {
            debug!("Connecting to CARL...");

            let mut carl = CarlClient::create(&host, port, &ca_certs, &client_auth, &domain_name_override, settings).await
                .context(format!("Could not connect to CARL at '{host}:{port}'."))?;

            log_version_compatibility(&mut carl).await?;

            match config {
                ConnectAndStart::Service { peer_messaging_client, rx_peer_configuration_state, remote_address } => {
                    trace!("Starting to process messages from CARL...");
                    peer_messaging_client.process_messages_loop(
                        &mut carl,
                        rx_peer_configuration_state.clone(),
                        remote_address,
                        &on_connect_success,
                        &connect_cancel,
                    ).await?;
                }
                ConnectAndStart::CarlClient { out } => {
                    trace!("Connection check to CARL succeeded.");
                    if let Some(out) = out {
                        out.send(carl).await?;
                    }
                }
            }
            Ok(())
        }.await;

        match result {
            Ok(()) => match config {
                ConnectAndStart::Service { .. } => {
                    warn!("Connection to CARL was interrupted. Reconnecting...");
                }
                ConnectAndStart::CarlClient { .. } => {
                    return Ok(());
                }
            }
            Err(cause) => {
                if connect_cancel.is_cancelled() {
                    info!("Connection to CARL was explicitly cancelled. Terminating EDGAR.");
                    break;
                }
                else {
                    let mut backoff = backoff.lock().await;

                    if let Some(delay) = backoff.next() {
                        error!("Error in connection to CARL. Reconnecting in {delay:?}. Error was: {cause:?}");

                        tokio::select! {
                            _ = tokio::time::sleep(delay) => {}
                            _ = connect_cancel.cancelled() => return Ok(()),
                        }
                    } else {
                        error!("Error in connection to CARL. No retries left. Terminating EDGAR. Error was: {cause:?}");
                        break;
                    }
                }
            }
        };
    }

    Ok(())
}
pub enum ConnectAndStart<'a> {
    Service {
        peer_messaging_client: &'a PeerMessagingClient,
        rx_peer_configuration_state: Arc<Mutex<mpsc::Receiver<EdgePeerConfigurationState>>>,
        remote_address: IpAddr,
    },
    CarlClient {
        out: Option<mpsc::Sender<CarlClient>>,
    },
}


pub struct ConnectOptions {
    pub host: String,
    pub port: u16,
    pub ca_certs: Vec<Pem>,
    pub client_auth: ClientAuth,
    pub domain_name_override: Option<String>,
    pub retries: usize,
    pub interval: Duration,
}
impl ConnectOptions {
    pub fn load_from_config(settings: &LoadedConfig) -> anyhow::Result<Self> {
        let host = settings.get_string("network.carl.host")?;
        let port = u16::try_from(settings.get_int("network.carl.port")?)?;

        let ca_certs = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_CA, None, settings)
            .context("No CA certificates found in configured locations")?;

        let client_auth = ClientAuth::load_from_config_for_carl_connection(settings)
            .context("Error while loading configuration for client authentication")?;

        let domain_name_override = {
            let domain_name_override = settings.get_string("network.tls.domain.name.override")?;
            domain_name_override.is_empty().not().then_some(domain_name_override)
        };

        let retries = settings.get::<usize>("network.connect.retries")?;
        let interval = Duration::from_millis(u64::try_from(settings.get_int("network.connect.interval.ms")?)?);

        Ok(Self { host, port, ca_certs, client_auth, domain_name_override, retries, interval })
    }
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
