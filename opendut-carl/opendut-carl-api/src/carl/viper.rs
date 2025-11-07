#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_model::cluster::ClusterId;
use opendut_model::viper::{ViperRunId, ViperSourceId, ViperSourceName};
use opendut_model::format::{format_id_with_name, format_id_with_optional_name};


//
// ViperSourceDescriptor
//

#[derive(thiserror::Error, Debug)]
pub enum StoreViperSourceDescriptorError {
    #[error("Test suite source {source} could not be created, due to internal errors:\n  {cause}", source=format_id_with_name(source_id, source_name))]
    Internal {
        source_id: ViperSourceId,
        source_name: ViperSourceName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteViperSourceDescriptorError {
    #[error("Test suite source <{source_id}> could not be deleted, because a source with that ID does not exist!")]
    SourceNotFound {
        source_id: ViperSourceId,
    },
    #[error("Test suite source <{source_id}> could not be deleted, because a cluster deployment <{cluster_id}> using this source still exists!")]
    ClusterDeploymentExists {
        source_id: ViperSourceId,
        cluster_id: ClusterId,
    },
    #[error("Test suite source {source} deleted with internal errors:\n  {cause}", source=format_id_with_optional_name(source_id, source_name))]
    Internal {
        source_id: ViperSourceId,
        source_name: Option<ViperSourceName>,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetViperSourceDescriptorError {
    #[error("A test suite source with ID <{source_id}> could not be found!")]
    SourceNotFound {
        source_id: ViperSourceId
    },
    #[error("An internal error occurred searching for a test suite source with ID <{source_id}>:\n  {cause}")]
    Internal {
        source_id: ViperSourceId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListViperSourceDescriptorsError {
    #[error("An internal error occurred computing the list of test suite sources:\n  {cause}")]
    Internal {
        cause: String
    }
}


//
// ViperRunDescriptor
//

#[derive(thiserror::Error, Debug)]
pub enum StoreViperRunDescriptorError {
    #[error("Test suite run <{run_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteViperRunDescriptorError {
    #[error("Test suite run <{run_id}> could not be deleted, because a run with that ID does not exist!")]
    RunNotFound {
        run_id: ViperRunId,
    },
    #[error("Test suite run <{run_id}> could not be deleted, because a cluster deployment <{cluster_id}> using this run still exists!")]
    ClusterDeploymentExists {
        run_id: ViperRunId,
        cluster_id: ClusterId,
    },
    #[error("Test suite run <{run_id}> deleted with internal errors:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetViperRunDescriptorError {
    #[error("A test suite run with ID <{run_id}> could not be found!")]
    RunNotFound {
        run_id: ViperRunId
    },
    #[error("An internal error occurred searching for a test suite run with ID <{run_id}>:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListViperRunDescriptorsError {
    #[error("An internal error occurred computing the list of test suite runs:\n  {cause}")]
    Internal {
        cause: String
    }
}


//
// ViperRunDeployment
//

#[derive(thiserror::Error, Debug)]
pub enum StoreViperRunDeploymentError {
    #[error("Test suite run deployment <{run_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteViperRunDeploymentError {
    #[error("Test suite run deployment <{run_id}> could not be deleted, because a run deployment with that ID does not exist!")]
    RunDeploymentNotFound {
        run_id: ViperRunId,
    },
    #[error("Test suite run deployment <{run_id}> deleted with internal errors:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetViperRunDeploymentError {
    #[error("A test suite run deployment with ID <{run_id}> could not be found!")]
    RunDeploymentNotFound {
        run_id: ViperRunId
    },
    #[error("An internal error occurred searching for a test suite run deployment with ID <{run_id}>:\n  {cause}")]
    Internal {
        run_id: ViperRunId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListViperRunDeploymentsError {
    #[error("An internal error occurred computing the list of test suite run deployments:\n  {cause}")]
    Internal {
        cause: String
    }
}


#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use super::*;
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};
    use opendut_model::viper::{ViperRunDescriptor, ViperRunId, ViperSourceDescriptor, ViperSourceId};
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


        pub async fn store_viper_run_descriptor(&mut self, descriptor: ViperRunDescriptor) -> Result<ViperRunId, ClientError<StoreViperRunDescriptorError>> {

            let request = tonic::Request::new(test_manager::StoreViperRunDescriptorRequest {
                run: Some(descriptor.into()),
            });

            let response = self.inner.store_viper_run_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::store_viper_run_descriptor_response::Reply::Failure(failure) => {
                    let error = StoreViperRunDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::store_viper_run_descriptor_response::Reply::Success(success) => {
                    let run_id = extract!(success.run_id)?;
                    Ok(run_id)
                }
            }
        }


        pub async fn delete_viper_run_descriptor(&mut self, run_id: ViperRunId) -> Result<ViperRunId, ClientError<DeleteViperRunDescriptorError>> {

            let request = tonic::Request::new(test_manager::DeleteViperRunDescriptorRequest {
                run_id: Some(run_id.into()),
            });

            let response = self.inner.delete_viper_run_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::delete_viper_run_descriptor_response::Reply::Failure(failure) => {
                    let error = DeleteViperRunDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::delete_viper_run_descriptor_response::Reply::Success(success) => {
                    let run_id = extract!(success.run_id)?;
                    Ok(run_id)
                }
            }
        }

        pub async fn get_viper_run_descriptor(&mut self, run_id: ViperRunId) -> Result<ViperRunDescriptor, ClientError<GetViperRunDescriptorError>> {

            let request = tonic::Request::new(test_manager::GetViperRunDescriptorRequest {
                run_id: Some(run_id.into()),
            });

            let response = self.inner.get_viper_run_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::get_viper_run_descriptor_response::Reply::Failure(failure) => {
                    let error = GetViperRunDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::get_viper_run_descriptor_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.descriptor)?;
                    Ok(peer_descriptor)
                }
            }
        }

        pub async fn list_viper_run_descriptors(&mut self) -> Result<Vec<ViperRunDescriptor>, ClientError<ListViperRunDescriptorsError>> {

            let request = tonic::Request::new(test_manager::ListViperRunDescriptorsRequest {});

            let response = self.inner.list_viper_run_descriptors(request).await?
                .into_inner();

            match extract!(response.reply)? {
                test_manager::list_viper_run_descriptors_response::Reply::Failure(failure) => {
                    let error = ListViperRunDescriptorsError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                test_manager::list_viper_run_descriptors_response::Reply::Success(success) => {
                    Ok(success.runs.into_iter()
                        .map(ViperRunDescriptor::try_from)
                        .collect::<Result<Vec<_>, _>>()?
                    )
                }
            }
        }
    }
}
