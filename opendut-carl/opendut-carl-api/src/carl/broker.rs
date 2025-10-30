use std::collections::HashMap;
#[cfg(feature = "client")]
pub use client::*;

use opendut_model::peer::configuration::{PeerConfiguration, EdgePeerConfigurationState};

pub mod error {
    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct ListPeers { pub message: String }

    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct OpenStream { pub message: String }
}


#[derive(Debug, Clone)]
pub struct UpstreamMessage {
    pub context: Option<TracingContext>,
    pub payload: UpstreamMessagePayload,
}

#[derive(Debug, Clone)]
pub enum UpstreamMessagePayload {
    EdgePeerConfigurationState(EdgePeerConfigurationState),
    Ping,
}


#[derive(Debug)]
pub struct DownstreamMessage {
    pub context: Option<TracingContext>,
    pub payload: DownstreamMessagePayload,
}

#[derive(Debug, PartialEq)]
pub enum DownstreamMessagePayload {
    Pong,
    ApplyPeerConfiguration(Box<ApplyPeerConfiguration>),
    DisconnectNotice,
}

#[derive(Debug, PartialEq)]
pub struct ApplyPeerConfiguration {
    pub configuration: PeerConfiguration,
}

#[derive(Debug, Clone)]
pub struct TracingContext {
    pub values: HashMap<String, String>,
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

    use crate::carl::broker::{error, stream_header, DownstreamMessage};
    use opendut_model::peer::PeerId;
    use tonic::codegen::tokio_stream::StreamExt;
    use opendut_util::future::ExplicitSendFutureWrapper;
    use crate::carl::{GrpcDownstream, GrpcUpstream};

    #[derive(Clone, Debug)]
    pub struct PeerMessagingBroker<T> {
        inner: PeerMessagingBrokerClient<T>,
    }

    impl<T> PeerMessagingBroker<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
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
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                >>::Error: Into<StdError> + Send + Sync,
        {
            let inner_client = PeerMessagingBrokerClient::new(InterceptedService::new(inner, interceptor));
            PeerMessagingBroker {
                inner: inner_client
            }
        }
    }

    pub type Downstream = GrpcDownstream<DownstreamMessage>;
    pub type Upstream = GrpcUpstream;

    impl<T> PeerMessagingBroker<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {

        pub async fn open_stream(&mut self, id: PeerId, remote_address: &IpAddr, extra_headers: stream_header::ExtraHeaders) -> Result<(Downstream, Upstream), error::OpenStream> {
            let (tx, rx) = mpsc::channel(1024);

            let response = {
                let mut request = tonic::Request::new(ReceiverStream::new(rx));
                request.metadata_mut().insert(stream_header::ID, MetadataValue::from_str(&id.to_string()).unwrap());
                request.metadata_mut().insert(stream_header::REMOTE_HOST, MetadataValue::from_str(&remote_address.to_string()).unwrap());

                {
                    let stream_header::ExtraHeaders { client_version: version } = extra_headers;
                    if let Some(version) = version {
                        request.metadata_mut().insert(stream_header::CLIENT_VERSION, MetadataValue::from_str(&version.value).unwrap());
                    }
                }

                ExplicitSendFutureWrapper::from(
                    self.inner.open(request)
                )
                    .await
                    .map_err(|cause| error::OpenStream { message: format!("Error while opening stream: {cause}") })?
            };

            let inbound = response.into_inner()
                .map(|result| result.and_then(|message| {
                    DownstreamMessage::try_from(message)
                        .map_err(|cause| tonic::Status::invalid_argument(format!("Error while converting stream message in open_stream: {cause}")))
                }));

            Ok((GrpcDownstream::from(inbound), GrpcUpstream::from(tx)))
        }
    }
}

pub mod stream_header {
    pub const ID: &str = "id";
    pub const REMOTE_HOST: &str = "remote-host";
    pub const CLIENT_VERSION: &str = "client_version";

    #[derive(Debug, Default)]
    pub struct ExtraHeaders {
        /// Version string of the client connecting.
        pub client_version: Option<PeerVersion>,
    }

    #[derive(Debug)]
    pub struct PeerVersion { pub value: String }
}
