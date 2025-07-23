use std::collections::HashMap;
use opendut_carl_api::proto::services::cluster_manager::cluster_manager_server::{ClusterManager as ClusterManagerService, ClusterManagerServer};
use opendut_carl_api::proto::services::cluster_manager::{CreateClusterDescriptorRequest, CreateClusterDescriptorResponse, create_cluster_descriptor_response, CreateClusterDescriptorSuccess, DeleteClusterDescriptorRequest, DeleteClusterDescriptorResponse, delete_cluster_descriptor_response, DeleteClusterDescriptorSuccess, GetClusterDescriptorRequest, GetClusterDescriptorResponse, get_cluster_descriptor_response, GetClusterDescriptorSuccess, GetClusterDescriptorFailure, ListClusterDescriptorsRequest, ListClusterDescriptorsResponse, list_cluster_descriptors_response, ListClusterDescriptorsSuccess, StoreClusterDeploymentRequest, StoreClusterDeploymentResponse, store_cluster_deployment_response, StoreClusterDeploymentSuccess, DeleteClusterDeploymentRequest, DeleteClusterDeploymentResponse, delete_cluster_deployment_response, DeleteClusterDeploymentSuccess, GetClusterDeploymentRequest, GetClusterDeploymentResponse, get_cluster_deployment_response, GetClusterDeploymentSuccess, GetClusterDeploymentFailure, ListClusterDeploymentsRequest, ListClusterDeploymentsResponse, list_cluster_deployments_response, ListClusterDeploymentsSuccess, ListClusterPeerStatesRequest, ListClusterPeerStatesResponse, list_cluster_peer_states_response, ListClusterPeerStatesSuccess};
use opendut_types::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
use tonic::{Request, Response, Status};
use tracing::{error, trace};

use crate::manager::cluster_manager::delete_cluster_deployment::DeleteClusterDeploymentParams;
use crate::manager::cluster_manager::{ClusterManagerRef, ClusterPeerStates, CreateClusterDescriptorError, CreateClusterDescriptorParams, DeleteClusterDescriptorError, DeleteClusterDescriptorParams, DeleteClusterDeploymentError};
use crate::manager::grpc::error::LogApiErr;
use crate::manager::grpc::extract;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::MapErrToInner;

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

    pub fn into_grpc_service(self) -> super::web::CorsGrpcWeb<ClusterManagerServer<Self>> {
        super::web::enable(ClusterManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl ClusterManagerService for ClusterManagerFacade {
    #[tracing::instrument(skip_all, level="trace")]
    async fn create_cluster_descriptor(&self, request: Request<CreateClusterDescriptorRequest>) -> Result<Response<CreateClusterDescriptorResponse>, Status> {

        let request = request.into_inner();
        let cluster: ClusterDescriptor = extract!(request.cluster_descriptor)?;

        trace!("Received request to create cluster descriptor: {cluster:?}");

        let result =
            self.resource_manager.resources_mut(async |resources|
                resources.create_cluster_descriptor(CreateClusterDescriptorParams {
                    cluster_descriptor: cluster.clone(),
                })
            ).await
            .map_err_to_inner(|source| CreateClusterDescriptorError::Persistence {
                cluster_id: cluster.id,
                cluster_name: cluster.name,
                source: source.context("Persistence error in transaction for creating cluster descriptor"),
            })
            .log_api_err()
                .map_err(opendut_carl_api::carl::cluster::CreateClusterDescriptorError::from);

        let reply = match result {
            Ok(cluster_id) => create_cluster_descriptor_response::Reply::Success(
                CreateClusterDescriptorSuccess {
                    cluster_id: Some(cluster_id.into())
                }
            ),
            Err(error) => create_cluster_descriptor_response::Reply::Failure(error.into())
        };

        Ok(Response::new(CreateClusterDescriptorResponse {
            reply: Some(reply)
        }))
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_cluster_descriptor(&self, request: Request<DeleteClusterDescriptorRequest>) -> Result<Response<DeleteClusterDescriptorResponse>, Status> {

        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.cluster_id)?;

        trace!("Received request to delete cluster descriptor for cluster <{cluster_id}>.");

        let result =
            self.resource_manager.resources_mut(async |resources|
                resources.delete_cluster_descriptor(DeleteClusterDescriptorParams {
                    cluster_id,
                })
            ).await
            .map_err_to_inner(|source| DeleteClusterDescriptorError::Persistence {
                cluster_id,
                cluster_name: None,
                source: source.context("Persistence error in transaction for deleting cluster descriptor"),
            })
            .log_api_err()
            .map_err(opendut_carl_api::carl::cluster::DeleteClusterDescriptorError::from);

        let reply = match result {
            Ok(cluster_descriptor) => delete_cluster_descriptor_response::Reply::Success(
                DeleteClusterDescriptorSuccess {
                    cluster_descriptor: Some(cluster_descriptor.into())
                }
            ),
            Err(error) => delete_cluster_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeleteClusterDescriptorResponse {
            reply: Some(reply)
        }))
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn get_cluster_descriptor(&self, request: Request<GetClusterDescriptorRequest>) -> Result<Response<GetClusterDescriptorResponse>, Status> {

        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.id)?;

        trace!("Received request to get cluster descriptor for cluster <{cluster_id}>.");

        let configuration = self.cluster_manager.lock().await.get_cluster_descriptor(cluster_id).await
            .log_api_err()
            .map_err(|cause| Status::internal(cause.to_string()))?;

        let result = match configuration {
            Some(configuration) => get_cluster_descriptor_response::Result::Success(
                GetClusterDescriptorSuccess {
                    configuration: Some(configuration.into())
                }
            ),
            None => get_cluster_descriptor_response::Result::Failure(
                GetClusterDescriptorFailure {}
            )
        };

        Ok(Response::new(GetClusterDescriptorResponse {
            result: Some(result)
        }))
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn list_cluster_descriptors(&self, _: Request<ListClusterDescriptorsRequest>) -> Result<Response<ListClusterDescriptorsResponse>, Status> {
        trace!("Received request to list cluster descriptors.");

        let configurations = self.cluster_manager.lock().await.list_cluster_descriptor().await
            .log_api_err()
            .map_err(|cause| Status::internal(cause.to_string()))?;

        Ok(Response::new(ListClusterDescriptorsResponse {
            result: Some(list_cluster_descriptors_response::Result::Success(
                ListClusterDescriptorsSuccess {
                    configurations: configurations.into_iter().map(std::convert::Into::into).collect::<Vec<_>>()
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
            .map_err(opendut_carl_api::carl::cluster::StoreClusterDeploymentError::from);

        let reply = match result {
            Ok(cluster_id) => {
                store_cluster_deployment_response::Reply::Success(
                    StoreClusterDeploymentSuccess {
                        cluster_id: Some(cluster_id.into())
                    }
                )
            }
            Err(error) => store_cluster_deployment_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(StoreClusterDeploymentResponse {
            reply: Some(reply),
        }))
    }
    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_cluster_deployment(&self, request: Request<DeleteClusterDeploymentRequest>) -> Result<Response<DeleteClusterDeploymentResponse>, Status> {
        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.cluster_id)?;
        let vpn = self.cluster_manager.lock().await.vpn.clone();

        trace!("Received request to delete cluster deployment for cluster <{cluster_id}>.");

        let result = self.resource_manager.resources_mut(async |resources|
            resources.delete_cluster_deployment(DeleteClusterDeploymentParams { cluster_id, vpn }).await
        ).await
            .map_err_to_inner(|source| DeleteClusterDeploymentError::Persistence {
                cluster_id,
                cluster_name: None,
                source: source.context("Persistence error in transaction for deleting cluster deployment"),
            })
            .log_api_err()
            .map_err(opendut_carl_api::carl::cluster::DeleteClusterDeploymentError::from);

        let reply = match result {
            Ok(cluster_descriptor) => delete_cluster_deployment_response::Reply::Success(
                DeleteClusterDeploymentSuccess {
                    cluster_deployment: Some(cluster_descriptor.into())
                }
            ),
            Err(error) => delete_cluster_deployment_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeleteClusterDeploymentResponse {
            reply: Some(reply)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_cluster_deployment(&self, request: Request<GetClusterDeploymentRequest>) -> Result<Response<GetClusterDeploymentResponse>, Status> {

        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.id)?;

        trace!("Received request to get cluster deployment for cluster <{cluster_id}>.");

        let deployment = self.cluster_manager.lock().await.get_cluster_deployment(cluster_id).await
            .log_api_err()
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
            .log_api_err()
            .map_err(|cause| Status::internal(cause.to_string()))?;

        Ok(Response::new(ListClusterDeploymentsResponse {
            result: Some(list_cluster_deployments_response::Result::Success(
                ListClusterDeploymentsSuccess {
                    deployments: deployments.into_iter().map(std::convert::Into::into).collect::<Vec<_>>()
                }
            ))
        }))
    }

    async fn list_cluster_peer_states(&self, request: Request<ListClusterPeerStatesRequest>) -> Result<Response<ListClusterPeerStatesResponse>, Status> {
        let request = request.into_inner();
        let cluster_id: ClusterId = extract!(request.cluster_id)?;
        trace!("Received request to list cluster peers for cluster <{cluster_id}>.");
        let result: ClusterPeerStates = self.resource_manager.resources_mut(async |resources| {
            resources.list_cluster_peer_states(cluster_id).await
        }).await
            .map_err(|cause| Status::internal(cause.to_string()))?
            .map_err(|cause| Status::internal(cause.to_string()))?;
        
        let peer_states = result.peer_states.iter().map(|(peer_id, peer_state)| (peer_id.uuid.to_string(), peer_state.clone())).collect::<HashMap<_, _>>();
        
        let response = Response::new(ListClusterPeerStatesResponse {
            result: Some(list_cluster_peer_states_response::Result::Success(
                ListClusterPeerStatesSuccess {
                    peer_states: peer_states.into_iter().map(|(peer_id, peer_state)| (peer_id, peer_state.into())).collect::<HashMap<_, _>>(),
                }
            )),
        });
        Ok(response)
    }
}
