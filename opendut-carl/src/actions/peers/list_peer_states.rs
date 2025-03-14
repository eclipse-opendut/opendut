use std::collections::HashMap;
use crate::resources::manager::ResourcesManagerRef;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tracing::{debug, error, info};
use tracing::log::trace;
use opendut_carl_api::carl::peer::ListPeerStatesError;
use opendut_types::peer::state::PeerState;
use crate::persistence::error::{PersistenceError, PersistenceResult};
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
                    let peer_state = resources.get::<PeerState>(peer.id)? //TODO this is quite inefficient (PeerId+PeerState could be returned from SQL, but that requires changing the ResourcesManager API)
                        .unwrap_or_else(|| {
                            trace!("Peer <{peer_id}> has no associated PeerState. Listing it as if it was Down.", peer_id=peer.id);
                            PeerState::Down
                        });

                    let peer_state = (peer.id, peer_state);

                    Ok::<_, PersistenceError>(peer_state)
                })
                .collect::<Result<Vec<_>, _>>()?;

            let peer_states = maybe_peer_states
                .into_iter()
                .collect::<HashMap<_, _>>();

            PersistenceResult::Ok(peer_states)
        }).await
        .map_err(|cause| ListPeerStatesError::Internal { cause: cause.to_string() })?;


        info!("Successfully queried all peer states.");

        Ok(peer_states)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
