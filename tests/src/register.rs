use anyhow::anyhow;
use opendut_util::telemetry;
use opendut_types::peer::state::PeerState;
use crate::util;

#[tokio::test(flavor = "multi_thread")]
async fn register_edgar_carl() -> anyhow::Result<()> {
    let _ = telemetry::initialize_test_logging().await?;

    let carl_port = util::spawn_carl()?;

    let edgar_id = util::spawn_edgar(carl_port)?;

    let carl_client = util::spawn_carl_client(carl_port).await?;

    util::retry(|| async {
        let mut carl_client = carl_client.lock().await;

        let edgar_state = carl_client.peers.get_peer_state(edgar_id).await
            .map_err(|cause| backoff::Error::transient(cause.into()))?;

        match edgar_state {
            PeerState::Up { .. } => Ok(()),
            PeerState::Down => Err(backoff::Error::transient(anyhow!("No peers registered in time!")))
        }
    }).await?;

    Ok(())
}
