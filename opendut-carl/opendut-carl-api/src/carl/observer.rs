use std::collections::{HashMap, HashSet};
use std::time::Duration;
#[cfg(feature = "client")]
pub use client::*;
use opendut_types::peer::PeerId;
use opendut_types::peer::state::PeerConnectionState;

pub mod error {
    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct WaitPeerOnline { pub message: String }

    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct OpenStream { pub message: String }
}

#[derive(Debug, Clone)]
pub struct WaitForPeersOnlineRequest {
    pub peer_ids: HashSet<PeerId>,
    pub max_observation_duration: Duration,
    pub peers_may_not_yet_exist: bool,
}
impl WaitForPeersOnlineRequest {
    pub const MIN_OBSERVATION_TIME_SECONDS: u64 = 30;
    pub const MAX_OBSERVATION_TIME_SECONDS: u64 = 60 * 60;
}


#[derive(Debug, Clone)]
pub enum WaitForPeersOnlineResponseStatus {
    WaitForPeersOnlineSuccess,
    WaitForPeersOnlineFailure {
        reason: String,
    },
    WaitForPeersOnlinePending,
}

#[derive(Debug, Clone)]
pub struct WaitForPeersOnlineResponse {
    pub peers: HashMap<PeerId, PeerConnectionState>,
    pub status: WaitForPeersOnlineResponseStatus,
}

#[cfg(feature = "client")]
mod client {
    use std::collections::HashSet;
    use std::time::Duration;
    use crate::carl::observer::{error, WaitForPeersOnlineResponse};
    use crate::proto::services::observer_messaging_broker::observer_messaging_broker_client::ObserverMessagingBrokerClient;
    use opendut_types::peer::PeerId;
    use tonic::codegen::{Body, Bytes, InterceptedService, StdError};
    use tonic::codegen::tokio_stream::StreamExt;
    use crate::carl::GrpcStream;

    #[derive(Clone, Debug)]
    pub struct ObserverMessagingBroker<T> {
        inner: ObserverMessagingBrokerClient<T>,
    }

    impl<T> ObserverMessagingBroker<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: ObserverMessagingBrokerClient<T>) -> ObserverMessagingBroker<T> {
            ObserverMessagingBroker {
                inner
            }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ObserverMessagingBroker<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            let inner_client = ObserverMessagingBrokerClient::new(InterceptedService::new(inner, interceptor));
            ObserverMessagingBroker {
                inner: inner_client
            }
        }
    }
    
    pub type WaitForPeerOnlineResponseStream = tonic::Streaming<crate::proto::services::observer_messaging_broker::WaitForPeersOnlineResponse>;

    impl<T> ObserverMessagingBroker<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub async fn wait_peers_online(
            &mut self,
            peer_ids: HashSet<PeerId>,
            max_observation_duration: Duration,
            peers_may_not_yet_exist: bool,
        ) -> Result<GrpcStream<WaitForPeersOnlineResponse>, error::OpenStream> {

            let request = crate::carl::observer::WaitForPeersOnlineRequest {
                peer_ids,
                max_observation_duration,
                peers_may_not_yet_exist,
            };
            let proto_request: crate::proto::services::observer_messaging_broker::WaitForPeersOnlineRequest = request.into();
            let response = self.inner
                .wait_for_peers_online(proto_request)
                .await
                .map_err(|cause| error::OpenStream { message: format!("Error while opening stream: {cause}") })?;

            let inbound = response.into_inner()
                .map(|result| result.and_then(|element| {
                    WaitForPeersOnlineResponse::try_from(element)
                        .map_err(|cause| tonic::Status::invalid_argument(format!("Error while converting stream message in wait_peers_online: {cause}")))
                }));

            Ok(GrpcStream::from(inbound))
        }
    }
}
