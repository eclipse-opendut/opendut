use opendut_carl_api::carl::CarlClient;
use opendut_edgar::service::start::DefaultPeerConfigurationApplier;
use opendut_types::peer::PeerId;
use opendut_types::util::Port;
use opendut_util::settings::LoadedConfig;
use std::future::Future;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::info;

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

pub fn spawn_edgar(carl_port: Port) -> anyhow::Result<PeerId> {
    let peer_id = PeerId::random();

    let edgar_config = load_edgar_config(carl_port, peer_id)?;

    tokio::spawn(async move {
        opendut_edgar::service::start::create::<DefaultPeerConfigurationApplier>(peer_id, edgar_config).await
            .expect("EDGAR crashed")
    });
    Ok(peer_id)
}

pub async fn spawn_carl_client(carl_port: Port) -> anyhow::Result<Mutex<CarlClient>> {
    let peer_id = PeerId::random();

    let edgar_config = load_edgar_config(carl_port, peer_id)?;

    let carl_client = opendut_edgar::common::carl::connect(&edgar_config.config).await
        .expect("Failed to connect to CARL for state checks");

    Ok(Mutex::new(carl_client))
}

pub async fn retry<T, Fut>(assertion: impl FnMut() -> Fut) -> anyhow::Result<T>
where
    Fut: Future<Output=Result<T, backoff::Error<anyhow::Error>>> + Sized
{
    let backoff_policy = backoff::ExponentialBackoffBuilder::new()
        .with_max_elapsed_time(Some(Duration::from_secs(15)))
        .build();

    backoff::future::retry(backoff_policy, assertion).await
}

fn load_edgar_config(carl_port: Port, peer_id: PeerId) -> anyhow::Result<LoadedConfig> {

    let settings_overrides = config::Config::builder()
        .set_override(opendut_edgar::common::settings::key::peer::id, peer_id.to_string())?
        .set_override("network.carl.host", "localhost")?
        .set_override("network.carl.port", carl_port.0)?
        .set_override("network.connect.retries", 100)?
        .set_override("network.oidc.enabled", false)?
        .build()?;

    opendut_edgar::common::settings::load_with_overrides(settings_overrides)
}

fn select_free_port() -> Port {
    let socket = std::net::TcpListener::bind("localhost:0").unwrap(); // Port 0 requests a free port from the operating system
    Port(socket.local_addr().unwrap().port())
    //socket is dropped at the end of this method, which releases the bound port, allowing us to use it
}
