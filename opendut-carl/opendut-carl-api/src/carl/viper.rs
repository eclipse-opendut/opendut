#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_model::cluster::ClusterId;
use opendut_model::viper::{TestSuiteSourceId, TestSuiteSourceName};
use opendut_model::format::{format_id_with_name, format_id_with_optional_name};

#[derive(thiserror::Error, Debug)]
pub enum StoreTestSuiteSourceDescriptorError {
    #[error("Test suite source {source} could not be created, due to internal errors:\n  {cause}", source=format_id_with_name(source_id, source_name))]
    Internal {
        source_id: TestSuiteSourceId,
        source_name: TestSuiteSourceName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteTestSuiteSourceDescriptorError {
    #[error("Test suite source <{source_id}> could not be deleted, because a source with that ID does not exist!")]
    SourceNotFound {
        source_id: TestSuiteSourceId,
    },
    #[error("Test suite source <{source_id}> could not be deleted, because a cluster deployment <{cluster_id}> using this source still exists!")]
    ClusterDeploymentExists {
        source_id: TestSuiteSourceId,
        cluster_id: ClusterId,
    },
    #[error("Test suite source {source} deleted with internal errors:\n  {cause}", source=format_id_with_optional_name(source_id, source_name))]
    Internal {
        source_id: TestSuiteSourceId,
        source_name: Option<TestSuiteSourceName>,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetTestSuiteSourceDescriptorError {
    #[error("A test suite source with ID <{source_id}> could not be found!")]
    SourceNotFound {
        source_id: TestSuiteSourceId
    },
    #[error("An internal error occurred searching for a test suite source with ID <{source_id}>:\n  {cause}")]
    Internal {
        source_id: TestSuiteSourceId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListTestSuiteSourceDescriptorsError {
    #[error("An internal error occurred computing the list of test suite sources:\n  {cause}")]
    Internal {
        cause: String
    }
}


#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use super::*;
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};
    use opendut_model::viper::{TestSuiteSourceDescriptor, TestSuiteSourceId};
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


        pub async fn delete_test_suite_source_descriptor(&mut self, source_id: TestSuiteSourceId) -> Result<TestSuiteSourceId, ClientError<DeleteTestSuiteSourceDescriptorError>> {

            let request = tonic::Request::new(test_manager::DeleteTestSuiteSourceDescriptorRequest {
                source_id: Some(source_id.into()),
            });

            let response = self.inner.delete_test_suite_source_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::delete_test_suite_source_descriptor_response::Reply::Failure(failure) => {
                    let error = DeleteTestSuiteSourceDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::delete_test_suite_source_descriptor_response::Reply::Success(success) => {
                    let source_id = extract!(success.source_id)?;
                    Ok(source_id)
                }
            }
        }

        pub async fn get_test_suite_source_descriptor(&mut self, source_id: TestSuiteSourceId) -> Result<TestSuiteSourceDescriptor, ClientError<GetTestSuiteSourceDescriptorError>> {

            let request = tonic::Request::new(test_manager::GetTestSuiteSourceDescriptorRequest {
                source_id: Some(source_id.into()),
            });

            let response = self.inner.get_test_suite_source_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::get_test_suite_source_descriptor_response::Reply::Failure(failure) => {
                    let error = GetTestSuiteSourceDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::get_test_suite_source_descriptor_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.descriptor)?;
                    Ok(peer_descriptor)
                }
            }
        }

        pub async fn list_test_suite_source_descriptors(&mut self) -> Result<Vec<TestSuiteSourceDescriptor>, ClientError<ListTestSuiteSourceDescriptorsError>> {

            let request = tonic::Request::new(test_manager::ListTestSuiteSourceDescriptorsRequest {});

            let response = self.inner.list_test_suite_source_descriptors(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::list_test_suite_source_descriptors_response::Reply::Failure(failure) => {
                    let error = ListTestSuiteSourceDescriptorsError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::list_test_suite_source_descriptors_response::Reply::Success(success) => {
                    Ok(success.sources.into_iter()
                        .map(TestSuiteSourceDescriptor::try_from)
                        .collect::<Result<Vec<_>, _>>()?
                    )
                }
            }
        }
    }
}
