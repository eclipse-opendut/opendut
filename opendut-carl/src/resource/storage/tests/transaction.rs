use anyhow::anyhow;
use crate::resource::persistence;
use crate::resource::storage::tests::peer_descriptor::peer_descriptor;
use crate::resource::storage::ResourcesStorageApi;
use googletest::prelude::*;
use opendut_types::peer::PeerDescriptor;

#[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
#[tokio::test]
async fn should_rollback_from_an_error_during_a_transaction() -> anyhow::Result<()> {
    let db = persistence::testing::spawn_and_connect_resource_manager().await?;
    let resource_manager = db.resource_manager;

    let peer = peer_descriptor()?;
    let peer_id = peer.id;

    let result = resource_manager.get::<PeerDescriptor>(peer_id).await?;
    assert!(result.is_none());

    let error = resource_manager.resources_mut::<_, (), anyhow::Error>(async |resources| {
        resources.insert(peer_id, peer)?; //will be rolled back
        let result = resources.get::<PeerDescriptor>(peer_id)?;
        assert!(result.is_some());

        Err(anyhow!("Intentionally failing transaction!"))
    }).await;

    assert_that!(error, ok(err(anything())));

    let result = resource_manager.get::<PeerDescriptor>(peer_id).await?;
    assert!(result.is_none(), "Expected database to have been rolled back due to error raised in transaction.");

    Ok(())
}
