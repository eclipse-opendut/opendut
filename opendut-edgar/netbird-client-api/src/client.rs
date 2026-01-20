use std::path::PathBuf;
use std::time::Duration;
use backon::{ExponentialBuilder, Retryable};
use tonic::transport::Channel;
use tracing::{debug, error, info};

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
        let path = format!("unix://{}", socket_path().display());

        debug!("Connecting to NetBird Client process via Unix domain socket at '{path}'...");

        let connect = || {
            DaemonServiceClient::connect(path.clone())
        };

        let connect_result = connect
            .retry(
                ExponentialBuilder::default()
                    .without_max_times() //continue retrying indefinitely
                    .with_max_delay(Duration::from_secs(60))
            )
            .notify(|cause: &tonic::transport::Error, sleep_duration: Duration| {
                debug!("Trying to connect to NetBird client after waiting {sleep_duration:?}. Had failed to connect due to: {cause}");
            })
            .await;

        match connect_result {
            Ok(client) => {
                info!("Connected to NetBird Client process via Unix domain socket at '{path}'.");
                Ok(Self { inner: client })
            }
            Err(cause) => {
                error!("Error while connecting to NetBird Client process via Unix domain socket at '{path}': {cause}");
                Err(Error::transport(cause, format!("Failed to connect to NetBird Unix domain socket at '{path}'")))
            }
        }
    }

    pub async fn full_status(&mut self) -> Result<FullStatus> {
        let request = tonic::Request::new(StatusRequest {
            get_full_peer_status: true,
            wait_for_ready: Some(true),
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
