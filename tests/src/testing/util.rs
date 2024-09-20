use crate::testing::peer_configuration_listener::PeerConfigurationReceiver;
use opendut_types::peer::PeerId;
use opendut_types::util::Port;
use opendut_util::settings::LoadedConfig;
use std::future::Future;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::info;

pub async fn retry<T, Fut>(assertion: impl FnMut() -> Fut) -> anyhow::Result<T>
where
    Fut: Future<Output=Result<T, backoff::Error<anyhow::Error>>> + Sized
{
    let backoff_policy = backoff::ExponentialBackoffBuilder::new()
        .with_max_elapsed_time(Some(Duration::from_secs(15)))
        .build();

    backoff::future::retry(backoff_policy, assertion).await
}

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
        // ensure the development certificates are used
        // even if ~/.config/opendut/carl/config.toml is present with different values for the test environment in opendut-vm
        .set_override("network.tls.certificate", "resources/development/tls/insecure-development-carl.pem")?
        .set_override("network.tls.key", "resources/development/tls/insecure-development-carl.key")?
        .build()?;
    let carl_settings = opendut_carl::settings::load_with_overrides(carl_config_override)?;
    tokio::spawn(async {
        opendut_carl::create(carl_settings).await
            .expect("CARL crashed")
    });

    Ok(carl_port)
}

pub async fn spawn_edgar_with_default_behavior(peer_id: PeerId, carl_port: Port) -> anyhow::Result<()> {
    let receiver = spawn_edgar_with_peer_configuration_receiver(peer_id, carl_port).await?;

    opendut_edgar::testing::service::peer_configuration::spawn_peer_configurations_handler(receiver.inner).await.unwrap();
    Ok(())
}

pub async fn spawn_edgar_with_peer_configuration_receiver(peer_id: PeerId, carl_port: Port) -> anyhow::Result<PeerConfigurationReceiver> {
    let edgar_config = load_edgar_config(carl_port, peer_id)?;

    let (tx_peer_configuration, rx_peer_configuration) = mpsc::channel(100);
    tokio::spawn(async move {
        opendut_edgar::testing::service::start::run_stream_receiver(peer_id, edgar_config, tx_peer_configuration).await
            .expect("EDGAR crashed")
    });
    Ok(PeerConfigurationReceiver { inner: rx_peer_configuration })
}

pub(super) fn load_edgar_config(carl_port: Port, peer_id: PeerId) -> anyhow::Result<LoadedConfig> {

    let settings_overrides = config::Config::builder()
        .set_override(opendut_edgar::testing::settings::key::peer::id, peer_id.to_string())?
        .set_override("network.carl.host", "localhost")?
        .set_override("network.carl.port", carl_port.0)?
        .set_override("network.connect.retries", 100)?
        .set_override("network.oidc.enabled", false)?
        .build()?;

    opendut_edgar::testing::settings::load_with_overrides(settings_overrides)
}


fn select_free_port() -> Port {
    let socket = std::net::TcpListener::bind("localhost:0").unwrap(); // Port 0 requests a free port from the operating system
    Port(socket.local_addr().unwrap().port())
    //socket is dropped at the end of this method, which releases the bound port, allowing us to use it
}

