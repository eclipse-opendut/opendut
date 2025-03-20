use std::collections::HashMap;
use crate::resource::manager::ResourceManagerRef;
use opendut_types::peer::{PeerId};
use tracing::{debug, error};
use opendut_carl_api::carl::peer::{ListPeerStatesError};
use opendut_types::peer::state::{PeerConnectionState, PeerState};
use crate::manager::peer_manager;
use crate::resource::persistence::error::{PersistenceError, PersistenceResult};
use crate::resource::storage::ResourcesStorageApi;

pub struct ListPeerStatesParams {
    pub resource_manager: ResourceManagerRef,
}

#[tracing::instrument(skip_all, level="trace")]
pub async fn list_peer_states(params: ListPeerStatesParams) -> Result<HashMap<PeerId, PeerState>, ListPeerStatesError> {

    async fn inner(params: ListPeerStatesParams) -> Result<HashMap<PeerId, PeerState>, ListPeerStatesError> {

        let resource_manager = params.resource_manager;

        debug!("Querying all peer states.");
        let peer_states = resource_manager.resources(async |resources| {
            let peer_member_states = peer_manager::internal::list_peer_member_states(resources)?;
            let peer_connection_states = resources.list::<PeerConnectionState>()?;

            let peer_states = peer_member_states.into_iter()
                .map(|(peer_id, member)| {
                    let connection = peer_connection_states.get(&peer_id).cloned().unwrap_or_default();
                    let peer_state = PeerState {
                        connection,
                        member,
                    };
                    Ok::<_, PersistenceError>((peer_id, peer_state))
                    
                })
                .collect::<Result<HashMap<_, _>, _>>()?;

            PersistenceResult::Ok(peer_states)
        }).await
        .map_err(|cause| ListPeerStatesError::Internal { cause: cause.to_string() })?;


        debug!("Successfully queried all peer states.");

        Ok(peer_states)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
