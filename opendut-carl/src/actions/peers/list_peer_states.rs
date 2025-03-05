use std::collections::HashMap;
use crate::resources::manager::ResourcesManagerRef;
use opendut_types::peer::{PeerId};
use tracing::{debug, error, info};
use opendut_carl_api::carl::peer::{ListPeerStatesError};
use opendut_types::peer::state::{PeerConnectionState, PeerState};
use crate::actions;
use crate::actions::ListPeerMemberStatesParams;
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
        let peer_member_states = actions::list_peer_member_states(ListPeerMemberStatesParams { resources_manager: resources_manager.clone() }).await
            .map_err(|cause| ListPeerStatesError::Internal { cause: cause.to_string() })?;  // only persistence error possible
        
        let peer_states = resources_manager.resources(|resources| {
            let peer_states = peer_member_states.into_iter()
                .map(|(peer_id, peer_member_state)| {
                    // TODO: peer state is partially hold in memory (connection state) and partially hold in database (membership due to cluster assignment) 
                    // TODO: PeerState and PeerConnectionState do not have a field `id` and the list() of ResourcesStorageApi returns only a vector of the stored resources => no id field in listing all elements!
                    let peer_connection_state = resources.get::<PeerConnectionState>(peer_id)?.unwrap_or_default();
                    let peer_state = PeerState {
                        connection: peer_connection_state,
                        member: peer_member_state,
                    };
                    Ok::<_, PersistenceError>((peer_id, peer_state))
                    
                })
                .collect::<Result<HashMap<_, _>, _>>()?;

            PersistenceResult::Ok(peer_states)
        }).await
        .map_err(|cause| ListPeerStatesError::Internal { cause: cause.to_string() })?;


        info!("Successfully queried all peer states.");

        Ok(peer_states)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
