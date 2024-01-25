use std::time::Duration;

use config::Config;
use googletest::prelude::*;

use opendut_types::peer::PeerId;
use opendut_util::logging;

use crate::util;

#[tokio::test(flavor = "multi_thread")]
async fn register_edgar_carl() -> Result<()> {
    logging::initialize()?;

    let carl_port = util::select_free_port();
    log::info!("Running test with CARL port {carl_port}.");

    let carl_config_override = config::Config::builder()
        .set_override("network.bind.port", carl_port)?
        .set_override("network.remote.port", carl_port)?
        .set_override("vpn.enabled", false)?
        .build()?;
    let _ = tokio::spawn(async {
        opendut_carl::create(carl_config_override).await
            .expect("CARL crashed")
    });

    let settings_overrides = Config::builder()
        .set_override(opendut_edgar::common::settings::key::peer::id, PeerId::random().to_string())?
        .set_override("network.carl.port", carl_port)?
        .set_override("network.connect.retries", 100)?
        .build()?;
    let edgar_config_override = opendut_edgar::common::settings::load_with_overrides(settings_overrides).unwrap()
        .config;

    let assert_channel_config = edgar_config_override.clone();

    let _ = tokio::spawn(async {
        opendut_edgar::service::start::create(edgar_config_override).await
            .expect("EDGAR crashed")
    });

    let mut carl_client = opendut_edgar::common::carl::connect(&assert_channel_config).await
        .expect("Failed to connect to CARL for state checks");

    let retries = 5;
    let interval = Duration::from_millis(500);
    for retries_left in (0..retries).rev() {
        let peers = carl_client.broker.list_peers().await?;

        if peers.is_empty() {
            if retries_left > 0 {
                tokio::time::sleep(interval).await;
            } else {
                fail!("EDGAR did not register within {retries}*{} ms!", interval.as_millis())?;
            }
        } else {
            return Ok(());
        }
    }
    Ok(())
}
