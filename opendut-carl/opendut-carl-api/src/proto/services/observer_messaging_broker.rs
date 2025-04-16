use std::collections::{HashMap, HashSet};
use opendut_types::conversion;
use opendut_types::proto::ConversionErrorBuilder;
use opendut_types::proto::ConversionResult;
use opendut_types::proto::ConversionError;
use crate::carl;
use crate::carl::observer::WaitForPeersOnlineResponseStatus;

tonic::include_proto!("opendut.carl.services.observer_messaging_broker");

conversion! {
    type Model = carl::observer::WaitForPeersOnlineRequest;
    type Proto = WaitForPeersOnlineRequest;
    
    fn from(value: Model) -> Proto {
        let peer_ids: Vec<opendut_types::proto::peer::PeerId> = value.peer_ids.into_iter().map(|id| id.into()).collect::<Vec<_>>();
        let max_observation_duration = value.max_observation_duration.as_secs();
        WaitForPeersOnlineRequest {
            peer_ids,
            max_observation_duration,
            peers_may_not_yet_exist: value.peers_may_not_yet_exist,
        }
    }
    fn try_from(value: Proto) -> ConversionResult<Model> {
        let peer_ids = value.peer_ids.into_iter()
            .map(opendut_types::peer::PeerId::try_from).collect::<Result<HashSet<_>, ConversionError>>()?;
        if value.max_observation_duration < Model::MIN_OBSERVATION_TIME_SECONDS || value.max_observation_duration > Model::MAX_OBSERVATION_TIME_SECONDS {
            return Err(ErrorBuilder::message(
                format!("Requested observation duration of <{}> seconds is not allowed. Allowed range: {} to {} seconds.", value.max_observation_duration, Model::MIN_OBSERVATION_TIME_SECONDS, Model::MAX_OBSERVATION_TIME_SECONDS)
            ))
        }
        let requested_duration = std::time::Duration::from_secs(value.max_observation_duration);

        Ok(Model {
            peer_ids,
            max_observation_duration: requested_duration,
            peers_may_not_yet_exist: value.peers_may_not_yet_exist,
        })
    }
}


conversion! {
    type Model = carl::observer::WaitForPeersOnlineResponse;
    type Proto = WaitForPeersOnlineResponse;
    
    fn from(value: Model) -> Proto {
        let peer_connection_states = value.peers.into_iter()
            .map(|(peer_id, peer_state)| (peer_id.uuid.to_string(), peer_state.into()))
            .collect::<HashMap<String, opendut_types::proto::peer::PeerConnectionState>>();
        let status = match value.status {
            WaitForPeersOnlineResponseStatus::WaitForPeersOnlineSuccess => {
                wait_for_peers_online_response::Status::Success(WaitForPeersOnlineSuccess {})
            }
            WaitForPeersOnlineResponseStatus::WaitForPeersOnlineFailure { reason  } => {
                wait_for_peers_online_response::Status::Failure(WaitForPeersOnlineFailure { reason })
            }
            WaitForPeersOnlineResponseStatus::WaitForPeersOnlinePending => {
                wait_for_peers_online_response::Status::Pending(WaitForPeersOnlinePending {})
            }
        };
        Proto {
            peer_states: peer_connection_states,
            status: Some(status),
        }
    }
    
    fn try_from(value: Proto) -> ConversionResult<Model> {
        let peer_connection_states = value.peer_states.into_iter().map(|(peer_id, peer_connection_state)| {
            let peer_id = opendut_types::peer::PeerId::try_from(peer_id.as_str());
            let peer_state = opendut_types::peer::state::PeerConnectionState::try_from(peer_connection_state);
            match (peer_id, peer_state) {
                (Ok(peer_id), Ok(peer_state)) => {
                    Ok((peer_id, peer_state))
                }
                (_, _) => {
                    Err(ErrorBuilder::message("Invalid peer connection state"))
                }
            }
        }).collect::<Result<HashMap<_, _>, ConversionError>>()?;
        let proto_status = extract!(value.status)?;
        
        let status = match proto_status {
            wait_for_peers_online_response::Status::Success(_) => {
                WaitForPeersOnlineResponseStatus::WaitForPeersOnlineSuccess
            }
            wait_for_peers_online_response::Status::Failure(WaitForPeersOnlineFailure { reason} ) => { 
                WaitForPeersOnlineResponseStatus::WaitForPeersOnlineFailure { reason }
            }
            wait_for_peers_online_response::Status::Pending(_) => {
                WaitForPeersOnlineResponseStatus::WaitForPeersOnlinePending { }
            }
        };

        Ok(Model {
            peers: peer_connection_states,
            status,
        })
    }
}
