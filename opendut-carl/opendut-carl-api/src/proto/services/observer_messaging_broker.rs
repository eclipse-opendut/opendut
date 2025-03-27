use opendut_types::conversion;
use opendut_types::proto::ConversionErrorBuilder;
use opendut_types::proto::ConversionResult;
use opendut_types::proto::ConversionError;
use crate::carl;

tonic::include_proto!("opendut.carl.services.observer_messaging_broker");

conversion! {
    type Model = carl::observer::WaitForPeersOnlineResponse;
    type Proto = WaitForPeersOnlineResponse;
    
    fn from(value: Model) -> Proto {
        let result = match value {
            Model::WaitForPeersOnlineSuccess => {
                Some(wait_for_peers_online_response::Result::Success(WaitForPeersOnlineSuccess {}))
            }
            Model::WaitForPeersOnlineFailure => {
                Some(wait_for_peers_online_response::Result::Failure(WaitForPeersOnlineFailure {}))
            }
            Model::WaitForPeersOnlinePending => {
                Some(wait_for_peers_online_response::Result::Pending(WaitForPeersOnlinePending {}))
            }
        };
        Proto {
            result,
        }
    }
    
    fn try_from(value: Proto) -> ConversionResult<Model> {
        let result = extract!(value.result)?;
        let result = match result {
            wait_for_peers_online_response::Result::Success(_) => {
                Model::WaitForPeersOnlineSuccess {}
            }
            wait_for_peers_online_response::Result::Failure(_) => { 
                Model::WaitForPeersOnlineFailure {}
            }
            wait_for_peers_online_response::Result::Pending(_) => {
                Model::WaitForPeersOnlinePending {}
            }
        };

        Ok(result)
    }
}