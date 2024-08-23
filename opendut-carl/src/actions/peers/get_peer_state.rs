use crate::resources::manager::ResourcesManagerRef;
use opendut_carl_api::carl::peer::GetPeerStateError;
use opendut_types::peer::state::PeerState;
use opendut_types::peer::PeerId;
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
            resources.get::<PeerState>(peer_id)
        }).await
            .map_err(|cause| GetPeerStateError::Internal {  peer_id ,cause: cause.to_string() })?
            .ok_or(GetPeerStateError::PeerNotFound { peer_id })?;


        info!("Successfully queried state of peer with peer_id <{}>.", peer_id);

        Ok(peer_state)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
