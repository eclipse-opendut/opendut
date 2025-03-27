use std::time::Duration;
use opendut_carl_api::carl::CarlClient;
use opendut_carl_api::carl::observer::WaitForPeersOnlineResponse;
use opendut_types::peer::PeerId;
use opendut_types::proto::ConversionError;

pub mod peer_online;
pub mod cluster_peers_online;

async fn await_peers_online(carl: &mut CarlClient, peer_ids: Vec<PeerId>) -> crate::Result<()> {
    let mut response_stream = carl.observer.wait_peers_online(peer_ids.clone()).await
        .map_err(|cause| format!("Failed to get stream: {}", cause.message))?;

    let timeout_duration = Duration::from_secs(5);

    loop {
        let received = tokio::time::timeout(timeout_duration, response_stream.message()).await;

        match received {
            Ok(response_result) => {
                match response_result {
                    Ok(Some(response)) => {
                        let conversion_result: Result<WaitForPeersOnlineResponse, ConversionError> = response.try_into();
                        match conversion_result {
                            Ok(response) => {
                                println!("Response: {:?}", response);
                                match response {
                                    WaitForPeersOnlineResponse::WaitForPeersOnlineSuccess | WaitForPeersOnlineResponse::WaitForPeersOnlineFailure => {
                                        break;
                                    }
                                    WaitForPeersOnlineResponse::WaitForPeersOnlinePending => {}
                                }
                            }
                            Err(error) => {
                                println!("Failed to decode message from server: {:?}", error);
                            }
                        }
                    }
                    Ok(None) => {
                        println!("CARL disconnected!");
                        return Err(format!("CARL disconnected while waiting for peers <{:?}> to come online!", peer_ids));
                    }
                    Err(error) => {
                        println!("GRPC error status: {:?}", error);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            Err(_elapsed_timeout_error) => {
                println!("No message for {} ms", timeout_duration.as_millis());
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
    
    Ok(())
}