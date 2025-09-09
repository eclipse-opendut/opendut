use crate::resource::persistence::error::PersistenceError;
use opendut_model::peer::state::PeerMemberState;
use opendut_model::peer::PeerId;
use crate::manager::peer_manager::list_peer_member_states::ListPeerMemberStatesError;
use crate::resource::api::resources::Resources;

impl Resources<'_> {
    pub fn get_peer_member_state(&self, peer_id: PeerId) -> Result<Option<PeerMemberState>, PersistenceError> {
        let peer_member_states = self.list_peer_member_states()
            .map_err(|source| match source {
                ListPeerMemberStatesError::Persistence { source } => source,
            })?;
        Ok(peer_member_states.get(&peer_id).cloned())
    }
}
