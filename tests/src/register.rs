use anyhow::anyhow;
use opendut_util::telemetry;
use std::ops::Not;

use crate::util;

#[tokio::test(flavor = "multi_thread")]
async fn register_edgar_carl() -> anyhow::Result<()> {
    let _ = telemetry::initialize_test_logging().await?;

    let carl_port = util::spawn_carl()?;

    util::spawn_edgar(carl_port)?;

    let carl_client = util::spawn_carl_client(carl_port).await?;

    util::retry(|| async {
        let mut carl_client = carl_client.lock().await;

        let peers = carl_client.broker.list_peers().await
            .map_err(|cause| backoff::Error::transient(cause.into()))?;

        if peers.is_empty().not() {
            Ok(())
        } else {
            Err(backoff::Error::transient(anyhow!("No peers registered in time!")))
        }
    }).await?;

    Ok(())
}
