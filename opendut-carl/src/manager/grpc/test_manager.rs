use tonic::{Request, Response, Status};
use tracing::{error, trace};
use opendut_carl_api::proto::services::test_manager::{delete_viper_source_descriptor_response, get_viper_source_descriptor_response, list_viper_source_descriptors_response, store_viper_source_descriptor_response, DeleteViperSourceDescriptorRequest, DeleteViperSourceDescriptorResponse, DeleteViperSourceDescriptorSuccess, GetViperSourceDescriptorRequest, GetViperSourceDescriptorResponse, GetViperSourceDescriptorSuccess, ListViperSourceDescriptorsRequest, ListViperSourceDescriptorsResponse, ListViperSourceDescriptorsSuccess, StoreViperSourceDescriptorRequest, StoreViperSourceDescriptorResponse, StoreViperSourceDescriptorSuccess};
use opendut_carl_api::proto::services::test_manager::{delete_viper_run_descriptor_response, get_viper_run_descriptor_response, list_viper_run_descriptors_response, store_viper_run_descriptor_response, DeleteViperRunDescriptorRequest, DeleteViperRunDescriptorResponse, DeleteViperRunDescriptorSuccess, GetViperRunDescriptorRequest, GetViperRunDescriptorResponse, GetViperRunDescriptorSuccess, ListViperRunDescriptorsRequest, ListViperRunDescriptorsResponse, ListViperRunDescriptorsSuccess, StoreViperRunDescriptorRequest, StoreViperRunDescriptorResponse, StoreViperRunDescriptorSuccess};
use opendut_carl_api::proto::services::test_manager::{delete_viper_run_deployment_response, get_viper_run_deployment_response, list_viper_run_deployments_response, store_viper_run_deployment_response, DeleteViperRunDeploymentRequest, DeleteViperRunDeploymentResponse, DeleteViperRunDeploymentSuccess, GetViperRunDeploymentRequest, GetViperRunDeploymentResponse, GetViperRunDeploymentSuccess, ListViperRunDeploymentsRequest, ListViperRunDeploymentsResponse, ListViperRunDeploymentsSuccess, StoreViperRunDeploymentRequest, StoreViperRunDeploymentResponse, StoreViperRunDeploymentSuccess};
use opendut_carl_api::proto::services::test_manager::test_manager_server::{TestManager as TestManagerService, TestManagerServer};
use opendut_model::viper::{ViperRunDeployment, ViperRunDescriptor, ViperRunId, ViperSourceDescriptor, ViperSourceId};
use crate::manager::grpc::error::LogApiErr;
use crate::manager::grpc::extract;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::PersistenceError;

pub struct TestManagerFacade {
    pub resource_manager: ResourceManagerRef,
}

impl TestManagerFacade {
    pub fn into_grpc_service(self) -> super::web::CorsGrpcWeb<TestManagerServer<Self>> {
        super::web::enable(TestManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl TestManagerService for TestManagerFacade {

    //
    // ViperSourceDescriptor
    //

    #[tracing::instrument(skip_all, level="trace")]
    async fn store_viper_source_descriptor(&self, request: Request<StoreViperSourceDescriptorRequest>) -> Result<Response<StoreViperSourceDescriptorResponse>, Status> {

        let request = request.into_inner();
        let source: ViperSourceDescriptor = extract!(request.source)?;

        trace!("Received request to store test suite source descriptor: {source:?}");


        let result =
            self.resource_manager.insert(source.id, source.clone()).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::StoreViperSourceDescriptorError::Internal {
                    source_id: source.id,
                    source_name: source.name,
                    cause: String::from("Error when accessing persistence while storing test suite source descriptor"),
                });

        let reply = match result {
            Ok(()) => store_viper_source_descriptor_response::Reply::Success(
                StoreViperSourceDescriptorSuccess {
                    source_id: Some(source.id.into()),
                }
            ),
            Err(error) => store_viper_source_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(StoreViperSourceDescriptorResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_viper_source_descriptor(&self, request: Request<DeleteViperSourceDescriptorRequest>) -> Result<Response<DeleteViperSourceDescriptorResponse>, Status> {

        let request = request.into_inner();
        let source_id: ViperSourceId = extract!(request.source_id)?;

        trace!("Received request to delete test suite source descriptor for source <{source_id}>.");

        let result =
            self.resource_manager.remove::<ViperSourceDescriptor>(source_id).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::DeleteViperSourceDescriptorError::Internal {
                    source_id,
                    source_name: None,
                    cause: String::from("Error when accessing persistence while storing test suite source descriptor"),
                });

        let response = match result {
            Ok(_) => delete_viper_source_descriptor_response::Reply::Success(
                DeleteViperSourceDescriptorSuccess {
                    source_id: Some(source_id.into())
                }
            ),
            Err(error) => delete_viper_source_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeleteViperSourceDescriptorResponse {
            reply: Some(response),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_viper_source_descriptor(&self, request: Request<GetViperSourceDescriptorRequest>) -> Result<Response<GetViperSourceDescriptorResponse>, Status> {

        let request = request.into_inner();
        let source_id: ViperSourceId = extract!(request.source_id)?;

        trace!("Received request to get test suite source descriptor for source <{source_id}>.");

        let result =
            self.resource_manager.get::<ViperSourceDescriptor>(source_id).await
                .inspect_err(|error| error!("Error while getting test suite source descriptor from gRPC API: {error}"))
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::GetViperSourceDescriptorError::Internal {
                    source_id,
                    cause: String::from("Error when accessing persistence while getting test suite source descriptor"),
                });

        let response = match result {
            Ok(descriptor) => match descriptor {
                Some(descriptor) => get_viper_source_descriptor_response::Reply::Success(
                    GetViperSourceDescriptorSuccess {
                        descriptor: Some(descriptor.into())
                    }
                ),
                None => get_viper_source_descriptor_response::Reply::Failure(
                    opendut_carl_api::carl::viper::GetViperSourceDescriptorError::SourceNotFound { source_id }.into()
                ),
            }
            Err(error) => get_viper_source_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(GetViperSourceDescriptorResponse {
            reply: Some(response)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_viper_source_descriptors(&self, _: Request<ListViperSourceDescriptorsRequest>) -> Result<Response<ListViperSourceDescriptorsResponse>, Status> {

        trace!("Received request to list test suite source descriptors.");

        let result = self.resource_manager.list::<ViperSourceDescriptor>().await
            .inspect_err(|error| error!("Error while listing test suite source descriptors from gRPC API: {error}"))
            .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::ListViperSourceDescriptorsError::Internal {
                cause: String::from("Error when accessing persistence while listing test suite source descriptors"),
            });

        let response = match result {
            Ok(sources) => {
                let sources = sources.into_values()
                    .map(From::from)
                    .collect::<Vec<_>>();

                list_viper_source_descriptors_response::Reply::Success(
                    ListViperSourceDescriptorsSuccess { sources }
                )
            }
            Err(error) => list_viper_source_descriptors_response::Reply::Failure(error.into())
        };

        Ok(Response::new(ListViperSourceDescriptorsResponse {
            reply: Some(response)
        }))
    }



    //
    // ViperRunDescriptor
    //

    #[tracing::instrument(skip_all, level="trace")]
    async fn store_viper_run_descriptor(&self, request: Request<StoreViperRunDescriptorRequest>) -> Result<Response<StoreViperRunDescriptorResponse>, Status> {

        let request = request.into_inner();
        let run: ViperRunDescriptor = extract!(request.run)?;

        trace!("Received request to store test suite run descriptor: {run:?}");


        let result =
            self.resource_manager.insert(run.id, run.clone()).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::StoreViperRunDescriptorError::Internal {
                    run_id: run.id,
                    cause: String::from("Error when accessing persistence while storing test suite run descriptor"),
                });

        let reply = match result {
            Ok(()) => store_viper_run_descriptor_response::Reply::Success(
                StoreViperRunDescriptorSuccess {
                    run_id: Some(run.id.into()),
                }
            ),
            Err(error) => store_viper_run_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(StoreViperRunDescriptorResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_viper_run_descriptor(&self, request: Request<DeleteViperRunDescriptorRequest>) -> Result<Response<DeleteViperRunDescriptorResponse>, Status> {

        let request = request.into_inner();
        let run_id: ViperRunId = extract!(request.run_id)?;

        trace!("Received request to delete test suite run descriptor for run <{run_id}>.");

        let result =
            self.resource_manager.remove::<ViperRunDescriptor>(run_id).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::DeleteViperRunDescriptorError::Internal {
                    run_id,
                    cause: String::from("Error when accessing persistence while storing test suite run descriptor"),
                });

        let response = match result {
            Ok(_) => delete_viper_run_descriptor_response::Reply::Success(
                DeleteViperRunDescriptorSuccess {
                    run_id: Some(run_id.into())
                }
            ),
            Err(error) => delete_viper_run_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeleteViperRunDescriptorResponse {
            reply: Some(response),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_viper_run_descriptor(&self, request: Request<GetViperRunDescriptorRequest>) -> Result<Response<GetViperRunDescriptorResponse>, Status> {

        let request = request.into_inner();
        let run_id: ViperRunId = extract!(request.run_id)?;

        trace!("Received request to get test suite run descriptor for run <{run_id}>.");

        let result =
            self.resource_manager.get::<ViperRunDescriptor>(run_id).await
                .inspect_err(|error| error!("Error while getting test suite run descriptor from gRPC API: {error}"))
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::GetViperRunDescriptorError::Internal {
                    run_id,
                    cause: String::from("Error when accessing persistence while getting test suite run descriptor"),
                });

        let response = match result {
            Ok(descriptor) => match descriptor {
                Some(descriptor) => get_viper_run_descriptor_response::Reply::Success(
                    GetViperRunDescriptorSuccess {
                        descriptor: Some(descriptor.into())
                    }
                ),
                None => get_viper_run_descriptor_response::Reply::Failure(
                    opendut_carl_api::carl::viper::GetViperRunDescriptorError::RunNotFound { run_id }.into()
                ),
            }
            Err(error) => get_viper_run_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(GetViperRunDescriptorResponse {
            reply: Some(response)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_viper_run_descriptors(&self, _: Request<ListViperRunDescriptorsRequest>) -> Result<Response<ListViperRunDescriptorsResponse>, Status> {

        trace!("Received request to list test suite run descriptors.");

        let result = self.resource_manager.list::<ViperRunDescriptor>().await
            .inspect_err(|error| error!("Error while listing test suite run descriptors from gRPC API: {error}"))
            .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::ListViperRunDescriptorsError::Internal {
                cause: String::from("Error when accessing persistence while listing test suite run descriptors"),
            });

        let response = match result {
            Ok(runs) => {
                let runs = runs.into_values()
                    .map(From::from)
                    .collect::<Vec<_>>();

                list_viper_run_descriptors_response::Reply::Success(
                    ListViperRunDescriptorsSuccess { runs }
                )
            }
            Err(error) => list_viper_run_descriptors_response::Reply::Failure(error.into())
        };

        Ok(Response::new(ListViperRunDescriptorsResponse {
            reply: Some(response)
        }))
    }


    //
    // ViperRunDeployment
    //

    #[tracing::instrument(skip_all, level="trace")]
    async fn store_viper_run_deployment(&self, request: Request<StoreViperRunDeploymentRequest>) -> Result<Response<StoreViperRunDeploymentResponse>, Status> {

        let request = request.into_inner();
        let run: ViperRunDeployment = extract!(request.run)?;

        trace!("Received request to store test suite run deployment: {run:?}");


        let result =
            self.resource_manager.insert(run.id, run.clone()).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::StoreViperRunDeploymentError::Internal {
                    run_id: run.id,
                    cause: String::from("Error when accessing persistence while storing test suite run deployment"),
                });

        let reply = match result {
            Ok(()) => store_viper_run_deployment_response::Reply::Success(
                StoreViperRunDeploymentSuccess {
                    run_id: Some(run.id.into()),
                }
            ),
            Err(error) => store_viper_run_deployment_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(StoreViperRunDeploymentResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_viper_run_deployment(&self, request: Request<DeleteViperRunDeploymentRequest>) -> Result<Response<DeleteViperRunDeploymentResponse>, Status> {

        let request = request.into_inner();
        let run_id: ViperRunId = extract!(request.run_id)?;

        trace!("Received request to delete test suite run deployment for run <{run_id}>.");

        let result =
            self.resource_manager.remove::<ViperRunDeployment>(run_id).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::DeleteViperRunDeploymentError::Internal {
                    run_id,
                    cause: String::from("Error when accessing persistence while storing test suite run deployment"),
                });

        let response = match result {
            Ok(_) => delete_viper_run_deployment_response::Reply::Success(
                DeleteViperRunDeploymentSuccess {
                    run_id: Some(run_id.into())
                }
            ),
            Err(error) => delete_viper_run_deployment_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeleteViperRunDeploymentResponse {
            reply: Some(response),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_viper_run_deployment(&self, request: Request<GetViperRunDeploymentRequest>) -> Result<Response<GetViperRunDeploymentResponse>, Status> {

        let request = request.into_inner();
        let run_id: ViperRunId = extract!(request.run_id)?;

        trace!("Received request to get test suite run deployment for run <{run_id}>.");

        let result =
            self.resource_manager.get::<ViperRunDeployment>(run_id).await
                .inspect_err(|error| error!("Error while getting test suite run deployment from gRPC API: {error}"))
                .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::GetViperRunDeploymentError::Internal {
                    run_id,
                    cause: String::from("Error when accessing persistence while getting test suite run deployment"),
                });

        let response = match result {
            Ok(deployment) => match deployment {
                Some(deployment) => get_viper_run_deployment_response::Reply::Success(
                    GetViperRunDeploymentSuccess {
                        deployment: Some(deployment.into())
                    }
                ),
                None => get_viper_run_deployment_response::Reply::Failure(
                    opendut_carl_api::carl::viper::GetViperRunDeploymentError::RunDeploymentNotFound { run_id }.into()
                ),
            }
            Err(error) => get_viper_run_deployment_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(GetViperRunDeploymentResponse {
            reply: Some(response)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_viper_run_deployments(&self, _: Request<ListViperRunDeploymentsRequest>) -> Result<Response<ListViperRunDeploymentsResponse>, Status> {

        trace!("Received request to list test suite run deployments.");

        let result = self.resource_manager.list::<ViperRunDeployment>().await
            .inspect_err(|error| error!("Error while listing test suite run deployments from gRPC API: {error}"))
            .map_err(|_: PersistenceError| opendut_carl_api::carl::viper::ListViperRunDeploymentsError::Internal {
                cause: String::from("Error when accessing persistence while listing test suite run deployments"),
            });

        let response = match result {
            Ok(runs) => {
                let runs = runs.into_values()
                    .map(From::from)
                    .collect::<Vec<_>>();

                list_viper_run_deployments_response::Reply::Success(
                    ListViperRunDeploymentsSuccess { runs }
                )
            }
            Err(error) => list_viper_run_deployments_response::Reply::Failure(error.into())
        };

        Ok(Response::new(ListViperRunDeploymentsResponse {
            reply: Some(response)
        }))
    }
}
