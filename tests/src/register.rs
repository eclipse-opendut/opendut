use crate::testing::carl_client::TestCarlClient;
use crate::testing::util;
use crate::testing;

#[test_log::test(
    tokio::test(flavor = "multi_thread")
)]
async fn register_edgar_carl() -> anyhow::Result<()> {
    crate::install_crypto_provider();

    let carl_port = util::spawn_carl()?;

    let carl_client = TestCarlClient::connect(carl_port).await?;
    let edgar = testing::peer_descriptor::store_peer_descriptor(&carl_client).await?;

    util::spawn_edgar_with_default_behavior(edgar.id, carl_port).await?;

    carl_client.await_peer_up(edgar.id).await?;

    Ok(())
}
