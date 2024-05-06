use std::path::PathBuf;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tracing::{debug, info};
use url::Url;
use opendut_types::vpn::netbird::SetupKey;

use crate::error::{Error, Result};
use crate::proto::daemon::{DownRequest, FullStatus, LoginRequest, StatusRequest, UpRequest};
use crate::proto::daemon::daemon_service_client::DaemonServiceClient;

pub fn socket_path() -> PathBuf {
    PathBuf::from("/var/run/netbird.sock")
}

pub struct Client {
    inner: DaemonServiceClient<Channel>,
}

impl Client {
    pub async fn connect() -> Result<Self> {
        debug!("Connecting to NetBird Client process via Unix domain socket at '{}'...", socket_path().display());
        let ignored_uri = "http://[::]"; //Valid URI has to be specified, but will be ignored. Taken from this example: https://github.com/hyperium/tonic/blob/2325e3293b8a54f3412a8c9a5fcac064fa82db56/examples/src/uds/client.rs

        let channel = Endpoint::try_from(ignored_uri)
            .unwrap_or_else(|cause| panic!("Failed to create endpoint for static URL '{ignored_uri}': {cause}"))
            .connect_with_connector(tower::service_fn(|_: Uri| {
                UnixStream::connect(socket_path())
            })).await
            .map_err(|cause| Error::transport(cause, format!("Failed to connect to NetBird Unix domain socket at '{}'", socket_path().display())))?;

        let client = DaemonServiceClient::new(channel);

        info!("Connected to NetBird Client process via Unix domain socket at '{}'.", socket_path().display());
        Ok(Self {
            inner: client,
        })
    }

    pub async fn login(&mut self, setup_key: &SetupKey, management_url: &Url, mtu: u16) -> Result<()> {
        let request = tonic::Request::new(LoginRequest {
            setup_key: setup_key.uuid.to_string(),
            management_url: management_url.to_string(),
            wg_iface_mtu: i32::from(mtu),
            ..Default::default()
        });
        let _ = self.inner.login(request).await?; //ignore response, only relevant for login without Setup Key

        debug!("Logged NetBird Client into NetBird Management Service at '{}' with Setup-Key '{}'.", management_url, setup_key.uuid);
        Ok(())
    }

    pub async fn up(&mut self) -> Result<()> {
        let request = tonic::Request::new(UpRequest {});
        let _ = self.inner.up(request).await?;

        debug!("Successfully set NetBird Client to 'up'.");
        Ok(())
    }

    pub async fn full_status(&mut self) -> Result<FullStatus> {
        let request = tonic::Request::new(StatusRequest {
            get_full_peer_status: true,
        });

        let response = self.inner.status(request).await?;

        let status = response.into_inner().full_status.expect("Requested full status, but did not receive any, while checking NetBird client status.");
        Ok(status)
    }

    pub async fn check_is_up(&mut self) -> Result<bool> {
        let connected = self.full_status().await?
            .management_state.expect("Received no management state, while checking NetBird client status.")
            .connected;
        Ok(connected)
    }

    pub async fn down(&mut self) -> Result<()> {
        let request = tonic::Request::new(DownRequest {});
        let _ = self.inner.down(request).await?;
        Ok(())
    }
}
