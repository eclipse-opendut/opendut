#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

pub mod error {
    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct ListPeers { pub message: String }

    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct OpenStream { pub message: String }
}

#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use cfg_if::cfg_if;
    use tonic::codegen::{Body, Bytes, StdError};
    use opendut_types::peer::{PeerId};
    use opendut_types::proto::ConversionError;
    use crate::carl::broker::error;

    use crate::proto::services::peer_messaging_broker;
    use crate::proto::services::peer_messaging_broker::peer_messaging_broker_client::PeerMessagingBrokerClient;

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

        pub async fn list_peers(&mut self) -> Result<Vec<PeerId>, error::ListPeers> {
            let request = tonic::Request::new(peer_messaging_broker::ListPeersRequest {});

            match self.inner.list_peers(request).await {
                Ok(response) => {
                    let ids = response.into_inner().peers
                        .into_iter()
                        .map(TryFrom::try_from)
                        .collect::<Result<Vec<PeerId>, ConversionError>>()
                        .map_err(|cause| error::ListPeers { message: cause.to_string() })?;
                    Ok(ids)
                },
                Err(status) => {
                    Err(error::ListPeers { message: format!("gRPC failure: {status}") })
                },
            }
        }
    }

    cfg_if! {
        if #[cfg(feature = "client")] {
            use tokio::sync::mpsc;
            use std::str::FromStr;
            use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
            use tonic::metadata::MetadataValue;

            type Downstream = tonic::Streaming<peer_messaging_broker::Downstream>;
            type Upstream = mpsc::Sender<peer_messaging_broker::Upstream>;

            impl<T> PeerMessagingBroker<T>
            where T: tonic::client::GrpcService<tonic::body::BoxBody>,
                  T::Error: Into<StdError>,
                  T::ResponseBody: Body<Data=Bytes> + Send + 'static,
                  <T::ResponseBody as Body>::Error: Into<StdError> + Send,
            {

                pub async fn open_stream(&mut self, id: PeerId) -> Result<(Downstream, Upstream), error::OpenStream> {
                    let (tx, rx) = mpsc::channel(1024);

                    let response = {
                        let mut request = tonic::Request::new(ReceiverStream::new(rx));
                        request.metadata_mut().insert("id", MetadataValue::from_str(&id.to_string()).unwrap());

                        self.inner
                            .open(request)
                            .await
                            .map_err(|cause| error::OpenStream { message: format!("Error while opening stream: {cause}") })?
                    };

                    let inbound = response.into_inner();

                    Ok((inbound, tx))
                }
            }
        }
    }
}
