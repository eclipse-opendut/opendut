use opendut_edgar::testing::service::peer_configuration::ApplyPeerConfigurationParams;
use opendut_model::peer::configuration::{PeerConfiguration, EdgePeerConfigurationState};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::error::Elapsed;
use tokio::time::timeout;

pub struct PeerConfigurationReceiver {
    pub inner: mpsc::Receiver<ApplyPeerConfigurationParams>,
    pub tx_peer_configuration_state: mpsc::Sender<EdgePeerConfigurationState>,
}
impl PeerConfigurationReceiver {
    pub async fn receive_peer_configuration(&mut self) -> anyhow::Result<PeerConfiguration> {
        let result = timeout(Duration::from_secs(10), self.inner.recv()).await
            .expect("Timeout while expecting peer configuration")
            .expect("Channel closed prematurely.");

        Ok(result.peer_configuration)
    }

    pub async fn expect_no_peer_configuration(&mut self) {
        let received = timeout(Duration::from_secs(3), self.inner.recv()).await;

        match received {
            Ok(Some(config)) => panic!("Received peer configuration despite expecting none to arrive: {config:?}"),
            Ok(None) => panic!("Channel closed prematurely."),
            Err(Elapsed { .. }) => {} //success
        }
    }
}
