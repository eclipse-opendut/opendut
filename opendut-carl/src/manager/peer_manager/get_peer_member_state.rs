pub mod internal {
    use crate::manager::peer_manager;
    use crate::resource::persistence::error::PersistenceError;
    use crate::resource::storage::ResourcesStorageApi;
    use opendut_types::peer::state::PeerMemberState;
    use opendut_types::peer::PeerId;

    pub fn get_peer_member_state(resources: &impl ResourcesStorageApi, peer_id: &PeerId) -> Result<Option<PeerMemberState>, PersistenceError> {
        let peer_member_states = peer_manager::internal::list_peer_member_states(resources)?;
        Ok(peer_member_states.get(peer_id).cloned())
    }
}