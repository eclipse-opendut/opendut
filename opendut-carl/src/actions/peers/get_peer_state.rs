use crate::resources::manager::ResourcesManagerRef;
use opendut_carl_api::carl::peer::GetPeerStateError;
use opendut_types::peer::state::PeerState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tracing::{debug, error, info};
use crate::resources::storage::ResourcesStorageApi;

pub struct GetPeerStateParams {
    pub peer: PeerId,
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn get_peer_state(params: GetPeerStateParams) -> Result<PeerState, GetPeerStateError> {

    async fn inner(params: GetPeerStateParams) -> Result<PeerState, GetPeerStateError> {

        let peer_id = params.peer;
        let resources_manager = params.resources_manager;

        debug!("Querying state of peer with peer_id <{}>.", peer_id);
        
        let peer_state = resources_manager.resources_mut(|resources| {
            let peer_state = resources.get::<PeerState>(peer_id)
                .map_err(|cause| GetPeerStateError::Internal { peer_id, cause: cause.to_string() })?;
            match peer_state {
                Some(peer_state) => { Ok(peer_state) }
                None => {
                    let peer_descriptor = resources.get::<PeerDescriptor>(peer_id)
                        .map_err(|cause| GetPeerStateError::Internal { peer_id, cause: cause.to_string() })?;
                    match peer_descriptor {
                        Some(_) => { Ok(PeerState::Down)  }
                        None => { Err(GetPeerStateError::PeerNotFound { peer_id }) }
                    }
                }
            }
        }).await
        .map_err(|cause| GetPeerStateError::Internal { peer_id, cause: cause.to_string() })??;

        info!("Successfully queried state of peer with peer_id <{}>.", peer_id);

        Ok(peer_state)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

#[cfg(test)]
mod tests {
    use crate::actions;
    use crate::actions::{get_peer_state, GetPeerStateParams, StorePeerDescriptorParams};
    use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};
    use googletest::prelude::*;
    use opendut_carl_api::carl::peer::GetPeerStateError;
    use opendut_types::peer::state::PeerState;
    use opendut_types::peer::{PeerDescriptor, PeerId};
    use std::sync::Arc;
    use crate::actions::testing::PeerFixture;
    use crate::settings::vpn::Vpn;

    #[tokio::test]
    async fn should_get_peer_state_down_in_memory() -> anyhow::Result<()> {
        let resources_manager = ResourcesManager::new_in_memory();
        should_get_peer_state(resources_manager).await
    }

    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[tokio::test]
    async fn should_get_peer_state_down_in_database() -> anyhow::Result<()> {
        let db = crate::persistence::database::testing::spawn_and_connect_resources_manager().await?;
        should_get_peer_state(db.resources_manager).await
    }

    async fn should_get_peer_state(resources_manager: ResourcesManagerRef) -> anyhow::Result<()> {
        let peer = PeerFixture::new();
        actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: Vpn::Disabled,
            peer_descriptor: peer.descriptor,
        }).await?;

        assert_that!(resources_manager.get::<PeerState>(peer.id).await?.as_ref(), none());

        let peer_state = get_peer_state(GetPeerStateParams {
            peer: peer.id,
            resources_manager: Clone::clone(&resources_manager),
        }).await?;

        assert_that!(peer_state, eq(&PeerState::Down));
        assert_that!(resources_manager.get::<PeerState>(peer.id).await?.as_ref(), none());
        Ok(())
    }

    #[tokio::test]
    async fn should_throw_error_if_peer_not_found_in_memory() -> anyhow::Result<()> {
        let resources_manager = ResourcesManager::new_in_memory();
        should_throw_error_if_peer_not_found(resources_manager).await
    }

    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[tokio::test]
    async fn should_throw_error_if_peer_not_found_in_database() -> anyhow::Result<()> {
        let db = crate::persistence::database::testing::spawn_and_connect_resources_manager().await?;
        should_throw_error_if_peer_not_found(db.resources_manager).await
    }

    async fn should_throw_error_if_peer_not_found(resources_manager: ResourcesManagerRef) -> anyhow::Result<()> {
        let peer = PeerFixture::new();

        actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: Vpn::Disabled,
            peer_descriptor: peer.descriptor,
        }).await?;

        let not_existing_peer_id = PeerId::random();
        assert_that!(resources_manager.get::<PeerDescriptor>(not_existing_peer_id).await?.as_ref(), none());

        let peer_state_result = get_peer_state(GetPeerStateParams {
            peer: not_existing_peer_id,
            resources_manager: Clone::clone(&resources_manager),
        }).await;

        assert_that!(peer_state_result, err(eq(&GetPeerStateError::PeerNotFound { peer_id: not_existing_peer_id })));
        Ok(())
    }
}
