use crate::resource::manager::ResourceManagerRef;
use crate::resource::storage::ResourcesStorageApi;
use opendut_carl_api::carl::peer::GetPeerStateError;
use opendut_types::peer::state::{PeerConnectionState, PeerState};
use opendut_types::peer::PeerId;
use tracing::{debug, error, info};

pub struct GetPeerStateParams {
    pub peer: PeerId,
    pub resource_manager: ResourceManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn get_peer_state(params: GetPeerStateParams) -> Result<PeerState, GetPeerStateError> {

    async fn inner(params: GetPeerStateParams) -> Result<PeerState, GetPeerStateError> {

        let peer_id = params.peer;
        let resource_manager = params.resource_manager;
        debug!("Querying state of peer with peer_id <{}>.", peer_id);

        let peer_state: Result<PeerState, GetPeerStateError> = resource_manager.resources(async |resources| {
            let peer_member_state = resources.get_peer_member_state(peer_id)
                .map_err(|cause| GetPeerStateError::Internal { peer_id, cause: cause.to_string() })?
                .ok_or_else(|| GetPeerStateError::PeerNotFound { peer_id })?;
            let connection = resources.get::<PeerConnectionState>(peer_id)
                .map_err(|cause| GetPeerStateError::Internal { peer_id, cause: cause.to_string() })?  // only persistence error possible
                .unwrap_or_default();

            info!("Successfully queried state of peer with peer_id <{}>.", peer_id);

            Ok(PeerState {
                connection,
                member: peer_member_state.clone(),
            })
        }).await;

        peer_state
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

#[cfg(test)]
mod tests {
    use crate::manager::peer_manager;
    use crate::manager::peer_manager::{GetPeerStateParams, StorePeerDescriptorParams};
    use crate::manager::testing::PeerFixture;
    use crate::resource::manager::{ResourceManager, ResourceManagerRef};
    use crate::settings::vpn::Vpn;
    use googletest::prelude::*;
    use opendut_carl_api::carl::peer::GetPeerStateError;
    use opendut_types::peer::state::{PeerConnectionState, PeerMemberState, PeerState};
    use opendut_types::peer::{PeerDescriptor, PeerId};

    #[tokio::test]
    async fn should_get_peer_state_down_in_memory() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        should_get_peer_state(resource_manager).await
    }

    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[tokio::test]
    async fn should_get_peer_state_down_in_database() -> anyhow::Result<()> {
        let db = crate::resource::persistence::database::testing::spawn_and_connect_resource_manager().await?;
        should_get_peer_state(db.resource_manager).await
    }

    async fn should_get_peer_state(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
        let peer = PeerFixture::new();

        resource_manager.resources_mut(async |resources| {
            resources.store_peer_descriptor(StorePeerDescriptorParams {
                vpn: Vpn::Disabled,
                peer_descriptor: peer.descriptor,
            }).await
        }).await??;

        let peer_state = peer_manager::get_peer_state(GetPeerStateParams {
            peer: peer.id,
            resource_manager: Clone::clone(&resource_manager),
        }).await?;

        assert_that!(peer_state, eq(&PeerState { connection: PeerConnectionState::Offline, member: PeerMemberState::Available}));
        Ok(())
    }

    #[tokio::test]
    async fn should_throw_error_if_peer_not_found_in_memory() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        should_throw_error_if_peer_not_found(resource_manager).await
    }

    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[tokio::test]
    async fn should_throw_error_if_peer_not_found_in_database() -> anyhow::Result<()> {
        let db = crate::resource::persistence::database::testing::spawn_and_connect_resource_manager().await?;
        should_throw_error_if_peer_not_found(db.resource_manager).await
    }

    async fn should_throw_error_if_peer_not_found(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
        let peer = PeerFixture::new();

        resource_manager.resources_mut(async |resources| {
            resources.store_peer_descriptor(StorePeerDescriptorParams {
                vpn: Vpn::Disabled,
                peer_descriptor: peer.descriptor,
            }).await
        }).await??;

        let not_existing_peer_id = PeerId::random();
        assert_that!(resource_manager.get::<PeerDescriptor>(not_existing_peer_id).await?.as_ref(), none());

        let peer_state_result = peer_manager::get_peer_state(GetPeerStateParams {
            peer: not_existing_peer_id,
            resource_manager: Clone::clone(&resource_manager),
        }).await;

        assert_that!(peer_state_result, err(eq(&GetPeerStateError::PeerNotFound { peer_id: not_existing_peer_id })));
        Ok(())
    }
}
