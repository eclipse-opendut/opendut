use std::sync::Arc;

use tonic::{Request, Response, Status};
use tonic_web::CorsGrpcWeb;

use opendut_carl_api::carl::cluster::DeleteClusterDeploymentError;
use opendut_carl_api::proto::services::cluster_manager::*;
use opendut_carl_api::proto::services::cluster_manager::cluster_manager_server::{ClusterManager as ClusterManagerService, ClusterManagerServer};
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};

use crate::actions;
use crate::actions::{CreateClusterConfigurationParams, DeleteClusterConfigurationParams};
use crate::cluster::manager::ClusterManagerRef;
use crate::grpc::extract;
use crate::resources::manager::ResourcesManagerRef;

pub struct ClusterManagerFacade {
    cluster_manager: ClusterManagerRef,
    resources_manager: ResourcesManagerRef,
}

impl ClusterManagerFacade {

    pub fn new(cluster_manager: ClusterManagerRef, resources_manager: ResourcesManagerRef) -> Self {
        Self {
            cluster_manager,
            resources_manager
        }
    }

    pub fn into_grpc_service(self) -> CorsGrpcWeb<ClusterManagerServer<Self>> {
        tonic_web::enable(ClusterManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl ClusterManagerService for ClusterManagerFacade {

    async fn create_cluster_configuration(&self, request: Request<CreateClusterConfigurationRequest>) -> Result<Response<CreateClusterConfigurationResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let request = request.into_inner();
        let cluster_configuration: ClusterConfiguration = extract!(request.cluster_configuration)?;

        let result = actions::create_cluster_configuration(CreateClusterConfigurationParams {
            resources_manager: Arc::clone(&self.resources_manager),
            cluster_configuration,
        }).await;

        match result {
            Err(error) => {
                Ok(Response::new(CreateClusterConfigurationResponse {
                    reply: Some(create_cluster_configuration_response::Reply::Failure(error.into()))
                }))
            }
            Ok(cluster_id) => {
                Ok(Response::new(CreateClusterConfigurationResponse {
                    reply: Some(create_cluster_configuration_response::Reply::Success(
                        CreateClusterConfigurationSuccess {
                            cluster_id: Some(cluster_id.into())
                        }
                    ))
                }))
            }
        }
    }

    async fn delete_cluster_configuration(&self, request: Request<DeleteClusterConfigurationRequest>) -> Result<Response<DeleteClusterConfigurationResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.cluster_id)?;

        let result =
            actions::delete_cluster_configuration(DeleteClusterConfigurationParams {
                resources_manager: Arc::clone(&self.resources_manager),
                cluster_id,
            }).await;

        match result {
            Err(error) => {
                Ok(Response::new(DeleteClusterConfigurationResponse {
                    reply: Some(delete_cluster_configuration_response::Reply::Failure(error.into()))
                }))
            }
            Ok(cluster_configuration) => {
                Ok(Response::new(DeleteClusterConfigurationResponse {
                    reply: Some(delete_cluster_configuration_response::Reply::Success(
                        DeleteClusterConfigurationSuccess {
                            cluster_configuration: Some(cluster_configuration.into())
                        }
                    ))
                }))
            }
        }
    }

    async fn get_cluster_configuration(&self, request: Request<GetClusterConfigurationRequest>) -> Result<Response<GetClusterConfigurationResponse>, Status> {
        log::trace!("Received request: {:?}", request);
        match request.into_inner().id {
            None => {
                Err(Status::invalid_argument("ClusterId is required."))
            }
            Some(id) => {
                let id = ClusterId::try_from(id)
                    .map_err(|_| Status::invalid_argument("Invalid ClusterId."))?;
                let configuration = self.cluster_manager.find_configuration(id).await;
                match configuration {
                    Some(configuration) => {
                        Ok(Response::new(GetClusterConfigurationResponse {
                            result: Some(get_cluster_configuration_response::Result::Success(
                                GetClusterConfigurationSuccess {
                                    configuration: Some(configuration.into())
                                }
                            ))
                        }))
                    }
                    None => {
                        Ok(Response::new(GetClusterConfigurationResponse {
                            result: Some(get_cluster_configuration_response::Result::Failure(
                                GetClusterConfigurationFailure {}
                            ))
                        }))
                    }
                }
            }
        }
    }

    async fn list_cluster_configurations(&self, request: Request<ListClusterConfigurationsRequest>) -> Result<Response<ListClusterConfigurationsResponse>, Status> {
        log::trace!("Received request: {:?}", request);
        let configurations = self.cluster_manager.list_configuration().await;
        Ok(Response::new(ListClusterConfigurationsResponse {
            result: Some(list_cluster_configurations_response::Result::Success(
                ListClusterConfigurationsSuccess {
                    configurations: configurations.into_iter().map(|configuration| configuration.into()).collect::<Vec<_>>()
                }
            ))
        }))
    }

    

    async fn store_cluster_deployment(&self, request: Request<StoreClusterDeploymentRequest>) -> Result<Response<StoreClusterDeploymentResponse>, Status> {
        log::trace!("Received request: {:?}", request);
        match request.into_inner().deployment {
            None => {
                Err(Status::invalid_argument("ClusterDeployment is required."))
            }
            Some(deployment) => {
                let deployment = ClusterDeployment::try_from(deployment);
                self.cluster_manager.create_deployment(deployment.unwrap()).await.unwrap(); // TODO: Handle error

                Ok(Response::new(StoreClusterDeploymentResponse {
                    result: Some(store_cluster_deployment_response::Result::Success(StoreClusterDeploymentSuccess {}))
                }))
            }
        }
    }

    async fn delete_cluster_deployment(&self, request: Request<DeleteClusterDeploymentRequest>) -> Result<Response<DeleteClusterDeploymentResponse>, Status> {
        log::trace!("Received request: {:?}", request);
        match request.into_inner().id {
            None => {
                Ok(Response::new(DeleteClusterDeploymentResponse {
                    result: Some(delete_cluster_deployment_response::Result::Failure(
                        DeleteClusterDeploymentFailure {
                            reason: Some(delete_cluster_deployment_failure::Reason::ClusterIdRequired(DeleteClusterDeploymentFailureClusterIdRequired {}))
                        }
                    ))
                }))
            }
            Some(id) => {
                match ClusterId::try_from(id) {
                    Err(cause) => {
                        Ok(Response::new(DeleteClusterDeploymentResponse {
                            result: Some(delete_cluster_deployment_response::Result::Failure(
                                DeleteClusterDeploymentFailure {
                                    reason: Some(delete_cluster_deployment_failure::Reason::InvalidClusterId(DeleteClusterDeploymentFailureInvalidClusterId {
                                        cause: cause.to_string()
                                    }))
                                }
                            ))
                        }))
                    }
                    Ok(id) => {
                        match self.cluster_manager.delete_deployment(id).await {
                            Err(DeleteClusterDeploymentError::ClusterNotFound { id }) => {
                                Ok(Response::new(DeleteClusterDeploymentResponse {
                                    result: Some(delete_cluster_deployment_response::Result::Failure(
                                        DeleteClusterDeploymentFailure {
                                            reason: Some(delete_cluster_deployment_failure::Reason::ClusterNotFound(DeleteClusterDeploymentFailureNotFound {
                                                id: Some(id.into())
                                            }))
                                        }
                                    ))
                                }))
                            }
                            Err(DeleteClusterDeploymentError::InvalidClusterId { cause }) => {
                                Ok(Response::new(DeleteClusterDeploymentResponse {
                                    result: Some(delete_cluster_deployment_response::Result::Failure(
                                        DeleteClusterDeploymentFailure {
                                            reason: Some(delete_cluster_deployment_failure::Reason::InvalidClusterId(DeleteClusterDeploymentFailureInvalidClusterId { cause }))
                                        }
                                    ))
                                }))
                            }
                            Ok(cluster_deployment) => {
                                Ok(Response::new(DeleteClusterDeploymentResponse {
                                    result: Some(delete_cluster_deployment_response::Result::Success(
                                        DeleteClusterDeploymentSuccess {
                                            deployment: Some(cluster_deployment.into())
                                        }
                                    ))
                                }))
                            }
                        }
                    }
                }
            }
        }
    }

    async fn list_cluster_deployments(&self, request: Request<ListClusterDeploymentsRequest>) -> Result<Response<ListClusterDeploymentsResponse>, Status> {
        log::trace!("Received request: {:?}", request);
        let deployments = self.cluster_manager.list_deployment().await;
        Ok(Response::new(ListClusterDeploymentsResponse {
            result: Some(list_cluster_deployments_response::Result::Success(
                ListClusterDeploymentsSuccess {
                    deployments: deployments.into_iter().map(|deployment| deployment.into()).collect::<Vec<_>>()
                }
            ))
        }))
    }
}
