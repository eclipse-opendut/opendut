use std::time::Duration;
use crate::testing::util;
use anyhow::anyhow;
use backon::Retryable;
use opendut_carl_api::carl::CarlClient;
use opendut_model::peer::state::PeerConnectionState;
use opendut_model::peer::PeerId;
use opendut_model::util::Port;
use tokio::sync::{Mutex, MutexGuard};

pub struct TestCarlClient {
    client: Mutex<CarlClient>,
}
impl TestCarlClient {

    pub async fn connect(carl_port: Port) -> anyhow::Result<Self> {
        let peer_id = PeerId::random();

        let edgar_config = util::load_edgar_config(carl_port, peer_id)?;

        let carl_client = opendut_edgar::testing::carl::connect(&edgar_config.config).await
            .expect("Failed to connect to CARL for state checks");

        let inner = Mutex::new(carl_client);
        Ok(TestCarlClient { client: inner })
    }

    pub async fn await_peer_up(&self, peer_id: PeerId) -> anyhow::Result<()> {

        (|| async {
            let edgar_state = self.inner().await
                .peers.get_peer_state(peer_id).await?;

            match edgar_state.connection {
                PeerConnectionState::Online { .. } => Ok(()),
                PeerConnectionState::Offline => Err(anyhow!("No peers registered in time!"))
            }
        })
            .retry(
                backon::ExponentialBuilder::default()
                    .with_max_delay(Duration::from_secs(15))
            )
            .await?;

        Ok(())
    }

    pub async fn inner(&self) -> MutexGuard<'_, CarlClient> {
        self.client.lock().await
    }
}
