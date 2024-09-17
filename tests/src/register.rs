use std::time::Duration;
use opendut_util::telemetry;

use crate::util;

#[tokio::test(flavor = "multi_thread")]
async fn register_edgar_carl() -> anyhow::Result<()> {
    let _ = telemetry::initialize_test_logging().await?;

    let carl_port = util::spawn_carl()?;

    util::spawn_edgar(carl_port)?;

    let mut carl_client = util::spawn_carl_client(carl_port).await?;

    let retries = 5;
    let interval = Duration::from_millis(500);
    for retries_left in (0..retries).rev() {
        let peers = carl_client.broker.list_peers().await?;

        if peers.is_empty() {
            if retries_left > 0 {
                tokio::time::sleep(interval).await;
            } else {
                panic!("EDGAR did not register within {retries}*{} ms!", interval.as_millis());
            }
        } else {
            return Ok(());
        }
    }
    Ok(())
}
