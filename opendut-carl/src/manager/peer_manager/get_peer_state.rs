use crate::resource::storage::ResourcesStorageApi;
use opendut_types::peer::state::{PeerConnectionState, PeerState};
use opendut_types::peer::PeerId;
use tracing::{debug, error, info};
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;


impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn get_peer_state(&self, peer_id: PeerId) -> Result<PeerState, GetPeerStateError> {

        debug!("Querying state of peer with peer_id <{}>.", peer_id);

        let peer_member_state = self.get_peer_member_state(peer_id)
            .map_err(|source| GetPeerStateError::Persistence { peer_id, source })?
            .ok_or_else(|| GetPeerStateError::PeerNotFound { peer_id })?;
        let connection = self.get::<PeerConnectionState>(peer_id)
            .map_err(|source| GetPeerStateError::Persistence { peer_id, source })?
            .unwrap_or_default();

        info!("Successfully queried state of peer with peer_id <{}>.", peer_id);

        Ok(PeerState {
            connection,
            member: peer_member_state.clone(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPeerStateError {
    #[error("A peer with id <{peer_id}> could not be found!")]
    PeerNotFound {
        peer_id: PeerId
    },
    #[error("Error when accessing persistence while getting peer state for peer <{peer_id}>")]
    Persistence {
        peer_id: PeerId,
        #[source] source: PersistenceError,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::manager::peer_manager::StorePeerDescriptorParams;
    use crate::manager::testing::PeerFixture;
    use crate::resource::manager::{ResourceManager, ResourceManagerRef};
    use crate::settings::vpn::Vpn;
    use googletest::prelude::*;
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

        let peer_state = resource_manager.resources(async |resources| {
            resources.get_peer_state(peer.id)
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

        let peer_state_result = resource_manager.resources(async |resources| {
            resources.get_peer_state(not_existing_peer_id)
        }).await;

        let Err(GetPeerStateError::PeerNotFound { peer_id }) = peer_state_result
        else { panic!("Result was not a PeerNotFound error.") };

        assert_eq!(peer_id, not_existing_peer_id);

        Ok(())
    }
}
