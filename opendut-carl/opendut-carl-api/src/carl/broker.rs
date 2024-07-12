#[cfg(feature = "client")]
pub use client::*;

pub mod error {
    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct ListPeers { pub message: String }

    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct OpenStream { pub message: String }
}

#[cfg(feature = "client")]
mod client {
    use tonic::codegen::{http, Body, Bytes, InterceptedService, StdError};

    use crate::proto::services::peer_messaging_broker::peer_messaging_broker_client::PeerMessagingBrokerClient;

    use std::net::IpAddr;

    use std::str::FromStr;
    use tokio::sync::mpsc;
    use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
    use tonic::metadata::MetadataValue;

    use crate::carl::broker::error;
    use crate::proto::services::peer_messaging_broker;
    use opendut_types::peer::PeerId;
    use opendut_util_core::future::ExplicitSendFutureWrapper;

    #[derive(Clone, Debug)]
    pub struct PeerMessagingBroker<T> {
        inner: PeerMessagingBrokerClient<T>,
    }

    impl<T> PeerMessagingBroker<T>
    where T: tonic::client::GrpcService<tonic::body::BoxBody>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: PeerMessagingBrokerClient<T>) -> PeerMessagingBroker<T> {
            PeerMessagingBroker {
                inner
            }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> PeerMessagingBroker<InterceptedService<T, F>>
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
            let inner_client = PeerMessagingBrokerClient::new(InterceptedService::new(inner, interceptor));
            PeerMessagingBroker {
                inner: inner_client
            }
        }
    }

    pub type Downstream = tonic::Streaming<peer_messaging_broker::Downstream>;
    pub type Upstream = mpsc::Sender<peer_messaging_broker::Upstream>;

    impl<T> PeerMessagingBroker<T>
    where T: tonic::client::GrpcService<tonic::body::BoxBody>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {

        pub async fn open_stream(&mut self, id: PeerId, remote_address: &IpAddr) -> Result<(Downstream, Upstream), error::OpenStream> {
            let (tx, rx) = mpsc::channel(1024);

            let response = {
                let mut request = tonic::Request::new(ReceiverStream::new(rx));
                request.metadata_mut().insert("id", MetadataValue::from_str(&id.to_string()).unwrap());
                request.metadata_mut().insert("remote-host", MetadataValue::from_str(&remote_address.to_string()).unwrap());

                ExplicitSendFutureWrapper::from(
                    self.inner
                        .open(request)
                )
                    .await
                    .map_err(|cause| error::OpenStream { message: format!("Error while opening stream: {cause}") })?
            };

            let inbound = response.into_inner();

            Ok((inbound, tx))
        }
    }
}
