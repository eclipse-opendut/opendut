use std::collections::HashMap;
use crate::resources::manager::ResourcesManagerRef;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tracing::{debug, error, info};
use tracing::log::trace;
use opendut_carl_api::carl::peer::ListPeerStatesError;
use opendut_types::peer::state::PeerState;
use crate::persistence::error::PersistenceError;
use crate::resources::storage::ResourcesStorageApi;

pub struct ListPeerStatesParams {
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip_all, level="trace")]
pub async fn list_peer_states(params: ListPeerStatesParams) -> Result<HashMap<PeerId, PeerState>, ListPeerStatesError> {

    async fn inner(params: ListPeerStatesParams) -> Result<HashMap<PeerId, PeerState>, ListPeerStatesError> {

        let resources_manager = params.resources_manager;

        debug!("Querying all peer states.");

        let peer_states = resources_manager.resources(|resources| {
            let peers = resources.list::<PeerDescriptor>()?;

            let maybe_peer_states = peers.into_iter()
                .map(|peer| {
                    let maybe_peer_state = resources.get::<PeerState>(peer.id)? //TODO this is quite inefficient (PeerId+PeerState could be returned from SQL, but that requires changing the ResourcesManager API)
                        .map(|peer_state| (peer.id, peer_state));

                    if maybe_peer_state.is_none() {
                        trace!("Peer <{peer_id}> has no associated PeerState. Skipping in list_peer_states().", peer_id=peer.id);
                    }

                    Ok::<_, PersistenceError>(maybe_peer_state)
                })
                .collect::<Result<Vec<Option<_>>, _>>()?;

            let peer_states = maybe_peer_states
                .into_iter()
                .flatten() //filter out all that are `None`
                .collect::<HashMap<_, _>>();

            Ok(peer_states)
        }).await
        .map_err(|cause| ListPeerStatesError::Internal { cause: cause.to_string() })?;


        info!("Successfully queried all peer states.");

        Ok(peer_states)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
