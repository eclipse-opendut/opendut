use crate::resources::manager::{ResourcesManagerRef};
use opendut_carl_api::carl::peer::{GetPeerStateError};
use opendut_types::peer::state::PeerState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tracing::{debug, error, info};

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
                .map_err(|cause| GetPeerStateError::Internal {  peer_id ,cause: cause.to_string() })?;
            match peer_state {
                Some(peer_state) => { Ok(peer_state) }
                None => {
                    match resources.get::<PeerDescriptor>(peer_id)
                        .map_err(|cause| GetPeerStateError::Internal {  peer_id ,cause: cause.to_string() })? {
                        Some(_) => { Ok(PeerState::Down)  }
                        None => { Err(GetPeerStateError::PeerNotFound { peer_id }) }
                    }
                }
            }
        }).await?;

        info!("Successfully queried state of peer with peer_id <{}>.", peer_id);

        Ok(peer_state)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use googletest::prelude::*;
    use rstest::rstest;
    use opendut_carl_api::carl::peer::GetPeerStateError;
    use opendut_types::peer::PeerId;
    use opendut_types::peer::state::PeerState;
    use crate::actions;
    use crate::actions::{GetPeerStateParams, StorePeerDescriptorOptions, StorePeerDescriptorParams};
    use crate::actions::peers::testing::{fixture, store_peer_descriptor_options, Fixture};
    use crate::resources::manager::ResourcesManager;

    #[rstest]
    #[tokio::test]
    async fn should_get_peer_state(fixture: Fixture, store_peer_descriptor_options: StorePeerDescriptorOptions) -> anyhow::Result<()> {

        let resources_manager = ResourcesManager::new_in_memory();

        actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: fixture.vpn,
            peer_descriptor: fixture.peer_a_descriptor,
            options: store_peer_descriptor_options,
        }).await?;
        
        let peer_state = actions::get_peer_state(GetPeerStateParams {
            peer: fixture.peer_a_id,
            resources_manager: Clone::clone(&resources_manager),
        }).await?;
        
        assert_that!(peer_state, eq(PeerState::Down));
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn should_not_find_peer_for_id(fixture: Fixture, store_peer_descriptor_options: StorePeerDescriptorOptions) -> anyhow::Result<()> {

        let resources_manager = ResourcesManager::new_in_memory();
        
        actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: fixture.vpn,
            peer_descriptor: fixture.peer_a_descriptor,
            options: store_peer_descriptor_options,
        }).await?;

        let peer_id = PeerId::random();
        let peer_state_result = actions::get_peer_state(GetPeerStateParams {
            peer: peer_id,
            resources_manager: Clone::clone(&resources_manager),
        }).await;

        assert_that!(peer_state_result, err(eq(GetPeerStateError::PeerNotFound { peer_id })));
        Ok(())
    }
}
