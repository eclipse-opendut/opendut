use std::path::PathBuf;

use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tracing::{debug, error, info};

use hyper_util::rt::TokioIo;

use crate::error::{Error, Result};
use crate::proto::daemon::daemon_service_client::DaemonServiceClient;
use crate::proto::daemon::{DownRequest, FullStatus, StatusRequest};

pub fn socket_path() -> PathBuf {
    PathBuf::from("/var/run/netbird.sock")
}

pub struct Client {
    inner: DaemonServiceClient<Channel>,
}

impl Client {
    pub async fn connect() -> Result<Self> {
        debug!("Connecting to NetBird Client process via Unix domain socket at '{}'...", socket_path().display());
        let ignored_uri = "http://[::]"; //Valid URI has to be specified, but will be ignored. Taken from this example: https://github.com/hyperium/tonic/blob/52a0f2f56cf578c7733d757aa548d23cee14c148/examples/src/uds/client.rs

        let channel_result = Endpoint::try_from(ignored_uri)
            .unwrap_or_else(|cause| panic!("Failed to create endpoint for static URL '{ignored_uri}': {cause}"))
            .connect_with_connector(tower::service_fn(|_: Uri| async {
                Ok::<_, std::io::Error>(TokioIo::new(
                    UnixStream::connect(socket_path()).await?
                ))
            })).await
            .map_err(|cause| Error::transport(cause, format!("Failed to connect to NetBird Unix domain socket at '{}'", socket_path().display())));

        match channel_result {
            Ok(channel) => {
                info!("Connected to NetBird Client process via Unix domain socket at '{}'.", socket_path().display());
                Ok(Self {
                    inner: DaemonServiceClient::new(channel),
                })
            }
            Err(cause) => {
                error!("Error while connecting to NetBird Client process via Unix domain socket at '{}': {cause}", socket_path().display());
                Err(cause)
            }
        }
    }

    pub async fn full_status(&mut self) -> Result<FullStatus> {
        let request = tonic::Request::new(StatusRequest {
            get_full_peer_status: true,
            ..Default::default()
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
