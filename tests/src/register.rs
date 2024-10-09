use crate::testing::carl_client::TestCarlClient;
use crate::testing::util;
use opendut_types::peer::PeerId;

#[test_log::test(
    tokio::test(flavor = "multi_thread")
)]
async fn register_edgar_carl() -> anyhow::Result<()> {
    let carl_port = util::spawn_carl()?;

    let edgar_id = PeerId::random();
    util::spawn_edgar_with_default_behavior(edgar_id, carl_port).await?;

    let carl_client = TestCarlClient::connect(carl_port).await?;

    carl_client.await_peer_up(edgar_id).await?;

    Ok(())
}
