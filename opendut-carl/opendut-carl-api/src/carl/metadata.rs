#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct VersionError {
    message: String,
}

#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};

    use opendut_types::proto::util::VersionInfo;

    use crate::carl::metadata::VersionError;
    use crate::proto::services::metadata_provider;
    use crate::proto::services::metadata_provider::metadata_provider_client::MetadataProviderClient;

    #[derive(Clone, Debug)]
    pub struct MetadataProvider<T> {
        inner: MetadataProviderClient<T>,
    }

    impl<T> MetadataProvider<T>
    where T: tonic::client::GrpcService<tonic::body::BoxBody>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: MetadataProviderClient<T>) -> MetadataProvider<T> {
            MetadataProvider { inner }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> MetadataProvider<InterceptedService<T, F>>
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
            let inner_client = MetadataProviderClient::new(InterceptedService::new(inner, interceptor));
            MetadataProvider {
                inner: inner_client
            }
        }

        pub async fn version(&mut self) -> Result<VersionInfo, VersionError> {
            let request = tonic::Request::new(metadata_provider::VersionRequest {});

            match self.inner.version(request).await {
                Ok(response) => {
                    let version = response.into_inner()
                        .version_info
                        .ok_or(VersionError { message: String::from("Response contains no version info!") })?;
                    Ok(version)
                },
                Err(status) => {
                    Err(VersionError { message: format!("gRPC failure: {status}") })
                },
            }
        }
    }
}
