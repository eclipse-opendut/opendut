#[cfg(feature = "client")]
pub use client::*;

pub mod error {
    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct WaitPeerOnline { pub message: String }

    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct OpenStream { pub message: String }
}

#[derive(Debug)]
pub enum WaitForPeersOnlineResponse {
    WaitForPeersOnlineSuccess,
    WaitForPeersOnlineFailure,
    WaitForPeersOnlinePending,
}

#[cfg(feature = "client")]
mod client {
    use crate::carl::observer::error;
    use crate::proto::services::observer_messaging_broker::observer_messaging_broker_client::ObserverMessagingBrokerClient;
    use opendut_types::peer::PeerId;
    use tonic::codegen::{Body, Bytes, InterceptedService, StdError};

    #[derive(Clone, Debug)]
    pub struct ObserverMessagingBroker<T> {
        inner: ObserverMessagingBrokerClient<T>,
    }

    impl<T> ObserverMessagingBroker<T>
    where T: tonic::client::GrpcService<tonic::body::BoxBody>,
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
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
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
    where T: tonic::client::GrpcService<tonic::body::BoxBody>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub async fn wait_peers_online(&mut self, peer_ids: Vec<PeerId>) -> Result<WaitForPeerOnlineResponseStream, error::OpenStream> {
            let peer_ids: Vec<opendut_types::proto::peer::PeerId> = peer_ids.into_iter().map(|peer_id| peer_id.into()).collect();
            let response = self.inner
                .wait_for_peers_online(crate::proto::services::observer_messaging_broker::WaitForPeersOnlineRequest { peer_ids })
                .await
                .map_err(|cause| error::OpenStream { message: format!("Error while opening stream: {cause}") })?;

            let inbound = response.into_inner();

            Ok(inbound)

        }
    }

}