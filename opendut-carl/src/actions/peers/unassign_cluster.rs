use tracing::debug;
use crate::persistence::error::PersistenceError;
use crate::resources::manager::ResourcesManagerRef;
use opendut_types::peer::state::{PeerState, PeerUpState};
use opendut_types::peer::PeerId;
use crate::resources::storage::ResourcesStorageApi;

pub struct UnassignClusterParams {
    pub resources_manager: ResourcesManagerRef,
    pub peer_id: PeerId,
}


#[derive(thiserror::Error, Debug)]
pub enum UnassignClusterError {
    #[error("Unassigning cluster for peer <{0}> failed, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("Error while persisting ClusterAssignment for peer <{peer_id}>.")]
    Persistence { peer_id: PeerId, #[source] source: PersistenceError },
}

pub async fn unassign_cluster(params: UnassignClusterParams) -> Result<(), UnassignClusterError> {
    let peer_id = params.peer_id;

    debug!("Unassigning cluster from peer <{peer_id}>.");

    params.resources_manager.resources_mut(|resources| {
        let peer_state = resources.get::<PeerState>(peer_id)
            .map_err(|source| UnassignClusterError::Persistence { peer_id, source })?
            .ok_or(UnassignClusterError::PeerNotFound(peer_id))?;

        match peer_state {
            PeerState::Down => {}
            PeerState::Up { remote_host, .. } => {
                resources.insert(peer_id, PeerState::Up {
                    inner: PeerUpState::Available,
                    remote_host,
                })
                .map_err(|source| UnassignClusterError::Persistence { peer_id, source })?;
            }
        }

        Ok(())
    }).await
    .map_err(|source| UnassignClusterError::Persistence { peer_id, source })??;

    Ok(())
}
