#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_model::test::suite::{TestSuiteSourceId, TestSuiteSourceName};

#[derive(thiserror::Error, Debug)]
pub enum StoreTestSuiteSourceDescriptorError {
    #[error("Test suite source descriptor '{source_name}' <{source_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        source_id: TestSuiteSourceId,
        source_name: TestSuiteSourceName,
        cause: String
    }
}

#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use super::*;
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};
    use opendut_model::test::suite::{TestSuiteSourceDescriptor, TestSuiteSourceId};
    use crate::carl::{extract, ClientError};
    use crate::proto::services::test_manager;
    use crate::proto::services::test_manager::test_manager_client::TestManagerClient;

    #[derive(Debug, Clone)]
    pub struct TestManager<T> {
        inner: TestManagerClient<T>,
    }

    impl<T> TestManager<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: TestManagerClient<T>) -> TestManager<T> {
            TestManager {
                inner
            }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> TestManager<InterceptedService<T, F>>
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
            let inner_client = TestManagerClient::new(InterceptedService::new(inner, interceptor));
            TestManager {
                inner: inner_client
            }
        }

        pub async fn store_test_suite_source_descriptor(&mut self, descriptor: TestSuiteSourceDescriptor) -> Result<TestSuiteSourceId, ClientError<StoreTestSuiteSourceDescriptorError>> {

            let request = tonic::Request::new(test_manager::StoreTestSuiteSourceDescriptorRequest {
                source: Some(descriptor.into()),
            });

            let response = self.inner.store_test_suite_source_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::store_test_suite_source_descriptor_response::Reply::Failure(failure) => {
                    let error = StoreTestSuiteSourceDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::store_test_suite_source_descriptor_response::Reply::Success(success) => {
                    let source_id = extract!(success.source_id)?;
                    Ok(source_id)
                }
            }
        }
    }
}
