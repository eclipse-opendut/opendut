use std::collections::HashSet;
use std::time::Duration;
use opendut_carl_api::carl::CarlClient;
use opendut_carl_api::carl::observer::{WaitForPeersOnlineResponse, WaitForPeersOnlineResponseStatus};
use opendut_types::peer::PeerId;
use opendut_types::proto::ConversionError;

pub mod peer_online;
pub mod cluster_peers_online;

async fn await_peers_online(carl: &mut CarlClient, peer_ids: HashSet<PeerId>, max_observation_duration: Duration, peers_may_not_yet_exist: bool) -> crate::Result<()> {
    let mut response_stream = carl.observer.wait_peers_online(peer_ids.clone(), max_observation_duration, peers_may_not_yet_exist).await
        .map_err(|cause| format!("Failed to get stream: {}", cause.message))?;

    let request_timeout_duration = Duration::from_secs(5);

    loop {
        let received = tokio::time::timeout(request_timeout_duration, response_stream.message()).await;

        match received {
            Ok(response_result) => {
                match response_result {
                    Ok(Some(response)) => {
                        let conversion_result: Result<WaitForPeersOnlineResponse, ConversionError> = response.try_into();
                        match conversion_result {
                            Ok(response) => {
                                println!("Response: {response:?}");
                                match response.status {
                                    WaitForPeersOnlineResponseStatus::WaitForPeersOnlineSuccess | WaitForPeersOnlineResponseStatus::WaitForPeersOnlineFailure { .. } => {
                                        break;
                                    }
                                    WaitForPeersOnlineResponseStatus::WaitForPeersOnlinePending => {}
                                }
                            }
                            Err(error) => {
                                println!("Failed to decode message from server: {error:?}");
                            }
                        }
                    }
                    Ok(None) => {
                        println!("CARL disconnected!");
                        return Err(format!("CARL disconnected while waiting for peers <{peer_ids:?}> to come online!"));
                    }
                    Err(error) => {
                        println!("GRPC error status: {error:?}");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            Err(_elapsed_timeout_error) => {
                println!("No message for {} ms.", request_timeout_duration.as_millis());
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
    Ok(())
}
