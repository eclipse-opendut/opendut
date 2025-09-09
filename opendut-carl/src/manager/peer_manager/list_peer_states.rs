use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::{PersistenceError, PersistenceResult};
use crate::resource::storage::ResourcesStorageApi;
use opendut_model::peer::state::{PeerConnectionState, PeerState};
use opendut_model::peer::PeerId;
use std::collections::HashMap;
use tracing::debug;
use crate::manager::peer_manager::list_peer_member_states::ListPeerMemberStatesError;

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn list_peer_states(&self) -> Result<HashMap<PeerId, PeerState>, ListPeerStatesError> {

        debug!("Querying all peer states.");
        let peer_states = (|| {
            let peer_member_states = self.list_peer_member_states()
                .map_err(|source| match source {
                    ListPeerMemberStatesError::Persistence { source } => source,
                })?;
            let peer_connection_states = self.list::<PeerConnectionState>()?;

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
        })()
        .map_err(|source| ListPeerStatesError::Persistence { source })?;

        debug!("Successfully queried all peer states.");

        Ok(peer_states)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListPeerStatesError {
    #[error("Error when accessing persistence while listing peer states")]
    Persistence {
        #[source] source: PersistenceError,
    }
}
