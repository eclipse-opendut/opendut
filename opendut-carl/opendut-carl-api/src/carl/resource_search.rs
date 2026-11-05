#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;
use opendut_model::topology::DeviceDescriptor;

#[derive(thiserror::Error, Debug)]
pub enum SearchError {
    #[error("An internal error occurred while searching for resources:\n  {cause}")]
    Internal {
        cause: String
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResourceMatch {
    Peer(PeerDescriptor),
    Cluster(ClusterDescriptor),
    Device(DeviceDescriptor),
}

#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use super::*;

    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};

    use crate::carl::{ClientError, extract};
    use crate::proto::services::resource_search;
    use crate::proto::services::resource_search::resource_search_client::ResourceSearchClient;

    #[derive(Clone, Debug)]
    pub struct ResourceSearcher<T> {
        inner: ResourceSearchClient<T>,
    }

    impl<T> ResourceSearcher<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: ResourceSearchClient<T>) -> ResourceSearcher<T> {
            ResourceSearcher { inner }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ResourceSearcher<InterceptedService<T, F>>
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
            let inner_client = ResourceSearchClient::new(InterceptedService::new(inner, interceptor));
            ResourceSearcher {
                inner: inner_client
            }
        }

        pub async fn search(&mut self, query: String) -> Result<Vec<ResourceMatch>, ClientError<SearchError>> {

            let request = tonic::Request::new(resource_search::SearchRequest { query });

            let response = self.inner.search(request).await?
                .into_inner();

            match extract!(response.reply)? {
                resource_search::search_response::Reply::Failure(failure) => {
                    let error = SearchError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                resource_search::search_response::Reply::Success(success) => {
                    success.matches.into_iter()
                        .map(|resource_match| {
                            let resource = resource_match.resource
                                .ok_or(ClientError::<SearchError>::InvalidResponse(String::from("resource field not set")))?;
                            let result = match resource {
                                resource_search::resource_match::Resource::Peer(peer) => {
                                    ResourceMatch::Peer(PeerDescriptor::try_from(peer)?)
                                }
                                resource_search::resource_match::Resource::Cluster(cluster) => {
                                    ResourceMatch::Cluster(ClusterDescriptor::try_from(cluster)?)
                                }
                                resource_search::resource_match::Resource::Device(device) => {
                                    ResourceMatch::Device(DeviceDescriptor::try_from(device)?)
                                }
                            };
                            Ok::<_, ClientError<SearchError>>(result)
                        })
                        .collect::<Result<Vec<_>, _>>()
                }
            }
        }
    }
}
