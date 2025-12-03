use crate::testing::peer_configuration_listener::PeerConfigurationReceiver;
use opendut_model::peer::PeerId;
use opendut_model::util::Port;
use opendut_util::settings::LoadedConfig;
use tokio::sync::mpsc;
use tracing::info;
use opendut_model::peer::configuration::EdgePeerConfigurationState;

#[tracing::instrument(name="test_spawn_carl")]
pub fn spawn_carl() -> anyhow::Result<Port> {
    let carl_port = select_free_port();
    info!("Running test with CARL port {carl_port}.");

    let carl_config_override = config::Config::builder()
        .set_override("network.bind.port", carl_port.0)?
        .set_override("network.remote.port", carl_port.0)?
        .set_override("network.remote.host", "localhost")?
        .set_override("vpn.enabled", false)?
        .set_override("serve.ui.presence_check", false)?
        .set_override("network.oidc.enabled", false)?
        .set_override("persistence.enabled", false)?
        // ensure the development certificates are used
        // even if ~/.config/opendut/carl/config.toml is present with different values for the test environment in opendut-vm
        .set_override("network.tls.certificate", "resources/development/tls/insecure-development-carl.pem")?
        .set_override("network.tls.key", "resources/development/tls/insecure-development-carl.key")?
        .set_override("network.tls.ca", "resources/development/tls/insecure-development-ca.pem")?
        .build()?;

    tokio::spawn(async {
        opendut_carl::create(
            carl_config_override,
            opendut_carl::StartupOptions {
                telemetry_enabled: false,
                ..Default::default()
            }
        ).await
            .expect("CARL crashed")
    });

    Ok(carl_port)
}

pub async fn spawn_edgar_with_default_behavior(peer_id: PeerId, carl_port: Port) -> anyhow::Result<()> {
    let receiver = spawn_edgar_with_peer_configuration_receiver(peer_id, carl_port).await?;

    opendut_edgar::testing::service::peer_configuration::spawn_peer_configurations_handler(receiver.inner, receiver.tx_peer_configuration_state).await.unwrap();
    Ok(())
}

pub async fn spawn_edgar_with_peer_configuration_receiver(peer_id: PeerId, carl_port: Port) -> anyhow::Result<PeerConfigurationReceiver> {
    let (tx_peer_configuration_state, rx_peer_configuration_state) = mpsc::channel::<EdgePeerConfigurationState>(100);

    let edgar_config = load_edgar_config(carl_port, peer_id)?;

    let (tx_peer_configuration, rx_peer_configuration) = mpsc::channel(100);
    tokio::spawn(async move {
        let carl = opendut_edgar::testing::carl::connect(&edgar_config.config).await
            .expect("Could not connect to CARL for spawning EDGAR");

        let mut peer_messaging_client = opendut_edgar::testing::service::peer_messaging_client::PeerMessagingClient::create(peer_id, carl, edgar_config, tx_peer_configuration)
            .await
            .expect("Could not create EDGAR peer messaging client");
        peer_messaging_client.process_messages_loop(rx_peer_configuration_state).await
            .expect("Could not communicate with CARL. EDGAR test instance.");
    });
    Ok(PeerConfigurationReceiver { inner: rx_peer_configuration, tx_peer_configuration_state })
}

pub(super) fn load_edgar_config(carl_port: Port, peer_id: PeerId) -> anyhow::Result<LoadedConfig> {

    let settings_overrides = config::Config::builder()
        .set_override(opendut_edgar::testing::settings::key::peer::id, peer_id.to_string())?
        .set_override("network.carl.host", "localhost")?
        .set_override("network.carl.port", carl_port.0)?
        .set_override("network.connect.retries", 20)?
        .set_override("network.oidc.enabled", false)?
        .build()?;

    opendut_edgar::testing::settings::load_with_overrides(settings_overrides)
}


fn select_free_port() -> Port {
    let socket = std::net::TcpListener::bind("localhost:0").unwrap(); // Port 0 requests a free port from the operating system
    Port(socket.local_addr().unwrap().port())
    //socket is dropped at the end of this method, which releases the bound port, allowing us to use it
}

