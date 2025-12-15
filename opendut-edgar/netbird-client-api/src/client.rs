use std::path::PathBuf;

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
    pub async fn connect() -> Result<Self> { //TODO this should perform a few retries, if the Netbird process isn't ready yet
        let path = format!("unix://{}", socket_path().display());

        debug!("Connecting to NetBird Client process via Unix domain socket at '{path}'...");

        match DaemonServiceClient::connect(path.clone()).await {
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
