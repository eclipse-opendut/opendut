use crate::testing::util;
use anyhow::anyhow;
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::state::PeerState;
use opendut_types::peer::PeerId;
use opendut_types::util::Port;
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
        util::retry(|| async {
            let edgar_state = self.inner().await.peers.get_peer_state(peer_id).await
                .map_err(|cause| backoff::Error::transient(cause.into()))?;

            match edgar_state {
                PeerState::Up { .. } => Ok(()),
                PeerState::Down => Err(backoff::Error::transient(anyhow!("No peers registered in time!")))
            }
        }).await?;
        Ok(())
    }

    pub async fn inner(&self) -> MutexGuard<CarlClient> {
        self.client.lock().await
    }
}
