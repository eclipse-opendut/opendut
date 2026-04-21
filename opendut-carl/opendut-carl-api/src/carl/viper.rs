#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_model::viper::{ViperTestId, ViperSourceId, ViperSourceName, ViperRunId};
use opendut_model::format::{format_id_with_name, format_id_with_optional_name};


//
// ViperSourceDescriptor
//

#[derive(thiserror::Error, Debug)]
pub enum StoreViperSourceDescriptorError {
    #[error("VIPER source {source} could not be created, due to internal errors:\n  {cause}", source=format_id_with_name(source_id, source_name))]
    Internal {
        source_id: ViperSourceId,
        source_name: ViperSourceName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteViperSourceDescriptorError {
    #[error("VIPER source <{source_id}> could not be deleted, because a source with that ID does not exist!")]
    SourceNotFound {
        source_id: ViperSourceId,
    },
    #[error("VIPER source <{source_id}> could not be deleted, because a test <{test_id}> using this source still exists!")]
    TestExists {
        source_id: ViperSourceId,
        test_id: ViperTestId,
    },
    #[error("VIPER source {source} deleted with internal errors:\n  {cause}", source=format_id_with_optional_name(source_id, source_name))]
    Internal {
        source_id: ViperSourceId,
        source_name: Option<ViperSourceName>,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetViperSourceDescriptorError {
    #[error("A VIPER source with ID <{source_id}> could not be found!")]
    SourceNotFound {
        source_id: ViperSourceId
    },
    #[error("An internal error occurred searching for a VIPER source with ID <{source_id}>:\n  {cause}")]
    Internal {
        source_id: ViperSourceId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListViperSourceDescriptorsError {
    #[error("An internal error occurred computing the list of VIPER sources:\n  {cause}")]
    Internal {
        cause: String
    }
}


//
// ViperTestSuiteParameters
//

#[derive(thiserror::Error, Debug)]
pub enum GetViperTestSuiteParametersError {
    #[error("A VIPER source with ID <{source_id}> could not be found, while fetching VIPER test suite descriptor.")]
    SourceNotFound {
        source_id: ViperSourceId,
    },
    #[error("Compilation failed while getting VIPER test suite descriptor for source <{source_id}>.")]
    Compilation {
        source_id: ViperSourceId,
    },
    #[error("Error while initializing VIPER runtime for VIPER test source <{source_id}>.")]
    ViperRuntime {
        source_id: ViperSourceId,
    },
    #[error("An internal error occurred while fetching the VIPER test suite descriptor for source <{source_id}>:\n  {cause}")]
    Internal {
        source_id: ViperSourceId,
        cause: String,
    }
}


//
// ViperTestRunDescriptor
//

#[derive(thiserror::Error, Debug)]
pub enum StoreViperTestRunDescriptorError {
    #[error("Test <{test_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        test_id: ViperTestId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteViperTestRunDescriptorError {
    #[error("Test <{test_id}> could not be deleted, because a test with that ID does not exist!")]
    TestNotFound {
        test_id: ViperTestId,
    },
    #[error("Test <{test_id}> could not be deleted, because a viper run deployment <{run_id}> using this test still exists!")]
    ViperRunDeploymentExists {
        test_id: ViperTestId,
        run_id: ViperRunId,
    },
    #[error("Test <{test_id}> deleted with internal errors:\n  {cause}")]
    Internal {
        test_id: ViperTestId,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetViperTestRunDescriptorError {
    #[error("A test with ID <{test_id}> could not be found!")]
    TestNotFound {
        test_id: ViperTestId
    },
    #[error("An internal error occurred searching for a test with ID <{test_id}>:\n  {cause}")]
    Internal {
        test_id: ViperTestId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListViperTestRunDescriptorsError {
    #[error("An internal error occurred computing the list of VIPER test descriptors:\n  {cause}")]
    Internal {
        cause: String
    }
}


//
// ViperRunDeployment
//

#[derive(thiserror::Error, Debug)]
pub enum StoreViperRunDeploymentError {
    #[error("VIPER run deployment <{run_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteViperRunDeploymentError {
    #[error("VIPER run deployment <{run_id}> could not be deleted, because a run deployment with that ID does not exist!")]
    RunDeploymentNotFound {
        run_id: ViperRunId,
    },
    #[error("VIPER run deployment <{run_id}> deleted with internal errors:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetViperRunDeploymentError {
    #[error("A VIPER run deployment with ID <{run_id}> could not be found!")]
    RunDeploymentNotFound {
        run_id: ViperRunId
    },
    #[error("An internal error occurred searching for a VIPER run deployment with ID <{run_id}>:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListViperRunDeploymentsError {
    #[error("An internal error occurred computing the list of VIPER run deployments:\n  {cause}")]
    Internal {
        cause: String
    }
}


#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use super::*;
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};
    use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperTestRunDescriptor, ViperTestId, ViperTestSuiteParameters};
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


        pub async fn store_viper_source_descriptor(&mut self, descriptor: ViperSourceDescriptor) -> Result<ViperSourceId, ClientError<StoreViperSourceDescriptorError>> {

            let request = tonic::Request::new(test_manager::StoreViperSourceDescriptorRequest {
                source: Some(descriptor.into()),
            });

            let response = self.inner.store_viper_source_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::store_viper_source_descriptor_response::Reply::Failure(failure) => {
                    let error = StoreViperSourceDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::store_viper_source_descriptor_response::Reply::Success(success) => {
                    let source_id = extract!(success.source_id)?;
                    Ok(source_id)
                }
            }
        }


        pub async fn delete_viper_source_descriptor(&mut self, source_id: ViperSourceId) -> Result<ViperSourceId, ClientError<DeleteViperSourceDescriptorError>> {

            let request = tonic::Request::new(test_manager::DeleteViperSourceDescriptorRequest {
                source_id: Some(source_id.into()),
            });

            let response = self.inner.delete_viper_source_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::delete_viper_source_descriptor_response::Reply::Failure(failure) => {
                    let error = DeleteViperSourceDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::delete_viper_source_descriptor_response::Reply::Success(success) => {
                    let source_id = extract!(success.source_id)?;
                    Ok(source_id)
                }
            }
        }

        pub async fn get_viper_source_descriptor(&mut self, source_id: ViperSourceId) -> Result<ViperSourceDescriptor, ClientError<GetViperSourceDescriptorError>> {

            let request = tonic::Request::new(test_manager::GetViperSourceDescriptorRequest {
                source_id: Some(source_id.into()),
            });

            let response = self.inner.get_viper_source_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::get_viper_source_descriptor_response::Reply::Failure(failure) => {
                    let error = GetViperSourceDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::get_viper_source_descriptor_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.descriptor)?;
                    Ok(peer_descriptor)
                }
            }
        }

        pub async fn list_viper_source_descriptors(&mut self) -> Result<Vec<ViperSourceDescriptor>, ClientError<ListViperSourceDescriptorsError>> {

            let request = tonic::Request::new(test_manager::ListViperSourceDescriptorsRequest {});

            let response = self.inner.list_viper_source_descriptors(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::list_viper_source_descriptors_response::Reply::Failure(failure) => {
                    let error = ListViperSourceDescriptorsError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::list_viper_source_descriptors_response::Reply::Success(success) => {
                    Ok(success.sources.into_iter()
                        .map(ViperSourceDescriptor::try_from)
                        .collect::<Result<Vec<_>, _>>()?
                    )
                }
            }
        }


        pub async fn get_viper_test_suite_parameters(&mut self, source_id: ViperSourceId) -> Result<ViperTestSuiteParameters, ClientError<GetViperTestSuiteParametersError>> {

            let request = tonic::Request::new(test_manager::GetViperTestSuiteParametersRequest {
                source_id: Some(source_id.into()),
            });

            let response = self.inner.get_viper_test_suite_parameters(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::get_viper_test_suite_parameters_response::Reply::Failure(failure) => {
                    let error = GetViperTestSuiteParametersError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::get_viper_test_suite_parameters_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.descriptor)?;
                    Ok(peer_descriptor)
                }
            }
        }


        pub async fn store_viper_test_run_descriptor(&mut self, descriptor: ViperTestRunDescriptor) -> Result<ViperTestId, ClientError<StoreViperTestRunDescriptorError>> {

            let request = tonic::Request::new(test_manager::StoreViperTestRunDescriptorRequest {
                test: Some(descriptor.into()),
            });

            let response = self.inner.store_viper_test_run_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::store_viper_test_run_descriptor_response::Reply::Failure(failure) => {
                    let error = StoreViperTestRunDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::store_viper_test_run_descriptor_response::Reply::Success(success) => {
                    let test_id = extract!(success.test_id)?;
                    Ok(test_id)
                }
            }
        }


        pub async fn delete_viper_test_run_descriptor(&mut self, test_id: ViperTestId) -> Result<ViperTestId, ClientError<DeleteViperTestRunDescriptorError>> {

            let request = tonic::Request::new(test_manager::DeleteViperTestRunDescriptorRequest {
                test_id: Some(test_id.into()),
            });

            let response = self.inner.delete_viper_test_run_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::delete_viper_test_run_descriptor_response::Reply::Failure(failure) => {
                    let error = DeleteViperTestRunDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::delete_viper_test_run_descriptor_response::Reply::Success(success) => {
                    let test_id = extract!(success.test_id)?;
                    Ok(test_id)
                }
            }
        }

        pub async fn get_viper_test_run_descriptor(&mut self, run_id: ViperTestId) -> Result<ViperTestRunDescriptor, ClientError<GetViperTestRunDescriptorError>> {

            let request = tonic::Request::new(test_manager::GetViperTestRunDescriptorRequest {
                test_id: Some(run_id.into()),
            });

            let response = self.inner.get_viper_test_run_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::get_viper_test_run_descriptor_response::Reply::Failure(failure) => {
                    let error = GetViperTestRunDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::get_viper_test_run_descriptor_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.descriptor)?;
                    Ok(peer_descriptor)
                }
            }
        }

        pub async fn list_viper_test_run_descriptors(&mut self) -> Result<Vec<ViperTestRunDescriptor>, ClientError<ListViperTestRunDescriptorsError>> {

            let request = tonic::Request::new(test_manager::ListViperTestRunDescriptorsRequest {});

            let response = self.inner.list_viper_test_run_descriptors(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::list_viper_test_run_descriptors_response::Reply::Failure(failure) => {
                    let error = ListViperTestRunDescriptorsError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::list_viper_test_run_descriptors_response::Reply::Success(success) => {
                    Ok(success.tests.into_iter()
                        .map(ViperTestRunDescriptor::try_from)
                        .collect::<Result<Vec<_>, _>>()?
                    )
                }
            }
        }
    }
}
