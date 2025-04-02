use opendut_carl_api::carl::cluster::{StoreClusterDeploymentError};
use opendut_carl_api::proto::services::cluster_manager::cluster_manager_server::{ClusterManager as ClusterManagerService, ClusterManagerServer};
use opendut_carl_api::proto::services::cluster_manager::*;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use tonic::{Request, Response, Status};
use tonic_web::CorsGrpcWeb;
use tracing::{error, trace, warn};

use crate::manager::cluster_manager::delete_cluster_deployment::DeleteClusterDeploymentParams;
use crate::manager::cluster_manager::{ClusterManagerRef, CreateClusterConfigurationError, CreateClusterConfigurationParams, DeleteClusterConfigurationError, DeleteClusterConfigurationParams};
use crate::manager::grpc::extract;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::MapToInner;

pub struct ClusterManagerFacade {
    cluster_manager: ClusterManagerRef,
    resource_manager: ResourceManagerRef,
}

impl ClusterManagerFacade {

    pub fn new(cluster_manager: ClusterManagerRef, resource_manager: ResourceManagerRef) -> Self {
        Self {
            cluster_manager,
            resource_manager
        }
    }

    pub fn into_grpc_service(self) -> CorsGrpcWeb<ClusterManagerServer<Self>> {
        tonic_web::enable(ClusterManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl ClusterManagerService for ClusterManagerFacade {
    #[tracing::instrument(skip_all, level="trace")]
    async fn create_cluster_configuration(&self, request: Request<CreateClusterConfigurationRequest>) -> Result<Response<CreateClusterConfigurationResponse>, Status> {

        let request = request.into_inner();
        let cluster: ClusterConfiguration = extract!(request.cluster_configuration)?;

        trace!("Received request to create cluster configuration: {cluster:?}");

        let result =
            self.resource_manager.resources_mut(async |resources|
                resources.create_cluster_configuration(CreateClusterConfigurationParams {
                    cluster_configuration: cluster.clone(),
                })
            ).await
            .map_to_inner(|source| CreateClusterConfigurationError::Persistence {
                cluster_id: cluster.id,
                cluster_name: cluster.name,
                source: source.context("Persistence error in transaction for creating cluster configuration"),
            })
            .inspect_err(|error| error!("{error}"));

        let response = match result {
            Ok(cluster_id) => create_cluster_configuration_response::Reply::Success(
                CreateClusterConfigurationSuccess {
                    cluster_id: Some(cluster_id.into())
                }
            ),
            Err(error) => create_cluster_configuration_response::Reply::Failure(
                opendut_carl_api::carl::cluster::CreateClusterConfigurationError::from(error).into()
            )
        };

        Ok(Response::new(CreateClusterConfigurationResponse {
            reply: Some(response)
        }))
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_cluster_configuration(&self, request: Request<DeleteClusterConfigurationRequest>) -> Result<Response<DeleteClusterConfigurationResponse>, Status> {

        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.cluster_id)?;

        trace!("Received request to delete cluster configuration for cluster <{cluster_id}>.");

        let result =
            self.resource_manager.resources_mut(async |resources|
                resources.delete_cluster_configuration(DeleteClusterConfigurationParams {
                    cluster_id,
                })
            ).await
            .map_to_inner(|source| DeleteClusterConfigurationError::Persistence {
                cluster_id,
                cluster_name: None,
                source: source.context("Persistence error in transaction for deleting cluster configuration"),
            })
            .inspect_err(|error| error!("{error}"));

        let response = match result {
            Ok(cluster_configuration) => delete_cluster_configuration_response::Reply::Success(
                DeleteClusterConfigurationSuccess {
                    cluster_configuration: Some(cluster_configuration.into())
                }
            ),
            Err(error) => delete_cluster_configuration_response::Reply::Failure(
                opendut_carl_api::carl::cluster::DeleteClusterConfigurationError::from(error).into()
            ),
        };

        Ok(Response::new(DeleteClusterConfigurationResponse {
            reply: Some(response)
        }))
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn get_cluster_configuration(&self, request: Request<GetClusterConfigurationRequest>) -> Result<Response<GetClusterConfigurationResponse>, Status> {

        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.id)?;

        trace!("Received request to get cluster configuration for cluster <{cluster_id}>.");

        let configuration = self.cluster_manager.lock().await.get_cluster_configuration(cluster_id).await
            .inspect_err(|error| error!("{error}"))
            .map_err(|cause| Status::internal(cause.to_string()))?;

        let response = match configuration {
            Some(configuration) => get_cluster_configuration_response::Result::Success(
                GetClusterConfigurationSuccess {
                    configuration: Some(configuration.into())
                }
            ),
            None => get_cluster_configuration_response::Result::Failure(
                GetClusterConfigurationFailure {}
            )
        };

        Ok(Response::new(GetClusterConfigurationResponse {
            result: Some(response)
        }))
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn list_cluster_configurations(&self, _: Request<ListClusterConfigurationsRequest>) -> Result<Response<ListClusterConfigurationsResponse>, Status> {
        trace!("Received request to list cluster configurations.");

        let configurations = self.cluster_manager.lock().await.list_cluster_configuration().await
            .inspect_err(|error| error!("{error}"))
            .map_err(|cause| Status::internal(cause.to_string()))?;

        Ok(Response::new(ListClusterConfigurationsResponse {
            result: Some(list_cluster_configurations_response::Result::Success(
                ListClusterConfigurationsSuccess {
                    configurations: configurations.into_iter().map(|configuration| configuration.into()).collect::<Vec<_>>()
                }
            ))
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn store_cluster_deployment(&self, request: Request<StoreClusterDeploymentRequest>) -> Result<Response<StoreClusterDeploymentResponse>, Status> {

        let request = request.into_inner();
        let cluster_deployment: ClusterDeployment = extract!(request.cluster_deployment)?;

        trace!("Received request to store cluster deployment: {cluster_deployment:?}");

        let result = self.cluster_manager.lock().await.store_cluster_deployment(cluster_deployment).await
            .inspect_err(|cause| error!("{cause}"))
            .map_err(StoreClusterDeploymentError::from);

        match result {
            Err(error) => {
                warn!("Error while storing cluster deployment: {error:?}");
                Ok(Response::new(StoreClusterDeploymentResponse {
                    reply: Some(store_cluster_deployment_response::Reply::Failure(error.into()))
                }))
            }
            Ok(cluster_id) => {
                Ok(Response::new(StoreClusterDeploymentResponse {
                    reply: Some(store_cluster_deployment_response::Reply::Success(
                        StoreClusterDeploymentSuccess {
                            cluster_id: Some(cluster_id.into())
                        }
                    ))
                }))
            }
        }
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_cluster_deployment(&self, request: Request<DeleteClusterDeploymentRequest>) -> Result<Response<DeleteClusterDeploymentResponse>, Status> {
        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.cluster_id)?;

        trace!("Received request to delete cluster deployment for cluster <{cluster_id}>.");

        let result = self.resource_manager.resources_mut(async |resources|
            resources.delete_cluster_deployment(DeleteClusterDeploymentParams { cluster_id }).await
        ).await;

        let response = match result {
            Ok(Ok(cluster_configuration)) => delete_cluster_deployment_response::Reply::Success(
                DeleteClusterDeploymentSuccess {
                    cluster_deployment: Some(cluster_configuration.into())
                }
            ),
            Ok(Err(error)) => delete_cluster_deployment_response::Reply::Failure(error.into()),
            Err(error) => {
                let cause = String::from("Error when handling transaction in database");
                error!("{cause}: {error}");

                delete_cluster_deployment_response::Reply::Failure(
                    opendut_carl_api::carl::cluster::DeleteClusterDeploymentError::Internal {
                        cluster_id,
                        cluster_name: None,
                        cause,
                    }.into()
                )
            }
        };

        Ok(Response::new(DeleteClusterDeploymentResponse {
            reply: Some(response)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_cluster_deployment(&self, request: Request<GetClusterDeploymentRequest>) -> Result<Response<GetClusterDeploymentResponse>, Status> {

        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.id)?;

        trace!("Received request to get cluster deployment for cluster <{cluster_id}>.");

        let deployment = self.cluster_manager.lock().await.get_cluster_deployment(cluster_id).await
            .map_err(|cause| Status::internal(cause.to_string()))?;

        match deployment {
            Some(configuration) => Ok(Response::new(GetClusterDeploymentResponse {
                result: Some(get_cluster_deployment_response::Result::Success(
                    GetClusterDeploymentSuccess {
                        deployment: Some(configuration.into())
                    }
                ))
            })),
            None => Ok(Response::new(GetClusterDeploymentResponse {
                result: Some(get_cluster_deployment_response::Result::Failure(
                    GetClusterDeploymentFailure {}
                ))
            }))
        }
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_cluster_deployments(&self, _: Request<ListClusterDeploymentsRequest>) -> Result<Response<ListClusterDeploymentsResponse>, Status> {
        trace!("Received request to list cluster deployments.");

        let deployments = self.cluster_manager.lock().await.list_cluster_deployment().await
            .map_err(|cause| Status::internal(cause.to_string()))?;

        Ok(Response::new(ListClusterDeploymentsResponse {
            result: Some(list_cluster_deployments_response::Result::Success(
                ListClusterDeploymentsSuccess {
                    deployments: deployments.into_iter().map(|deployment| deployment.into()).collect::<Vec<_>>()
                }
            ))
        }))
    }
}
