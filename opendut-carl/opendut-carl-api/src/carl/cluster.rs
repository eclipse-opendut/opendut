use std::collections::HashMap;
#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;
use opendut_model::cluster::{ClusterDisplay, ClusterId, ClusterName};
use opendut_model::cluster::state::ClusterState;
use opendut_model::peer::PeerId;
use opendut_model::peer::state::PeerState;
use opendut_model::ShortName;

#[derive(thiserror::Error, Debug)]
pub enum CreateClusterDescriptorError {
    #[error("ClusterConfigration '{cluster_name}' <{cluster_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DeleteClusterDescriptorError {
    #[error("ClusterDescriptor <{cluster_id}> could not be deleted, because a ClusterDeployment with that ID still exists!")]
    ClusterDeploymentFound {
        cluster_id: ClusterId
    },
    #[error("ClusterDescriptor <{cluster_id}> could not be deleted, because a ClusterDescriptor with that ID does not exist!")]
    ClusterDescriptorNotFound {
        cluster_id: ClusterId
    },
    #[error(
        "ClusterDescriptor '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{actual_state}'! A ClusterDescriptor can be deleted when cluster is in state: {required_states}",
        actual_state = actual_state.short_name(),
        required_states = ClusterState::short_names_joined(required_states),
    )]
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    #[error("ClusterDescriptor {cluster} deleted with internal errors:\n  {cause}", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
    Internal {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ClusterDescriptor <{cluster_id}> could not be retrieved:\n  {message}")]
pub struct GetClusterDescriptorError {
    pub cluster_id: ClusterId,
    pub message: String
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ListClusterDescriptorsError {
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum StoreClusterDeploymentError {
    #[error("ClusterDeployment for cluster {cluster} failed, due to down or already in use peers: {invalid_peers:?}", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
    IllegalPeerState {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        invalid_peers: Vec<PeerId>,
    },
    #[error("ClusterDeployment for cluster {cluster} could not be changed, due to internal errors:\n  {cause}", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
    Internal {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterDeploymentError {
    #[error("ClusterDeployment for cluster <{cluster_id}> could not be deleted, because a ClusterDeployment with that id does not exist!")]
    ClusterDeploymentNotFound {
        cluster_id: ClusterId
    },
    #[error(
        "ClusterDeployment for cluster '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{actual_state}'! A peer can be deleted when: {required_states}",
        actual_state = actual_state.short_name(),
        required_states = ClusterState::short_names_joined(required_states)
    )]
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    #[error("ClusterDeployment for cluster {cluster} deleted with internal errors:\n  {cause}", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
    Internal {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        cause: String
    },
}

#[derive(thiserror::Error, Debug)]
pub enum GetClusterDeploymentError {
    #[error("ClusterDeployment for cluster <{cluster_id}> could not be retrieved, due to internal errors:\n  {cause}")]
    Internal { cluster_id: ClusterId, cause: String },
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ListClusterDeploymentsError {
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ListClusterPeersError {
    pub message: String,
}

pub enum ListClusterPeerStatesResponse {
    Success {
        peer_states: HashMap<PeerId, PeerState>,
    },
    Failure {
        message: String,
    }
}

#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};
    use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
    use crate::carl::{ClientError, extract};
    use crate::proto::services::cluster_manager;
    use crate::proto::services::cluster_manager::cluster_manager_client::ClusterManagerClient;
    use super::*;

    #[derive(Clone, Debug)]
    pub struct ClusterManager<T> {
        inner: ClusterManagerClient<T>,
    }

    impl<T> ClusterManager<T>
        where T: tonic::client::GrpcService<tonic::body::Body>,
              T::Error: Into<StdError>,
              T::ResponseBody: Body<Data=Bytes> + Send + 'static,
              <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: ClusterManagerClient<T>) -> ClusterManager<T> {
            ClusterManager {
                inner
            }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ClusterManager<InterceptedService<T, F>>
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
            let inner_client = ClusterManagerClient::new(InterceptedService::new(inner, interceptor));
            ClusterManager {
                inner: inner_client
            }
        }

        pub async fn store_cluster_descriptor(&mut self, configuration: ClusterDescriptor) -> Result<ClusterId, ClientError<CreateClusterDescriptorError>> {

            let request = tonic::Request::new(cluster_manager::CreateClusterDescriptorRequest {
                cluster_descriptor: Some(configuration.into()),
            });

            let response = self.inner.create_cluster_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                cluster_manager::create_cluster_descriptor_response::Reply::Failure(failure) => {
                    let error = CreateClusterDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                cluster_manager::create_cluster_descriptor_response::Reply::Success(success) => {
                    let peer_id = extract!(success.cluster_id)?;
                    Ok(peer_id)
                }
            }
        }

        pub async fn delete_cluster_descriptor(&mut self, cluster_id: ClusterId) -> Result<ClusterDescriptor, ClientError<DeleteClusterDescriptorError>> {

            let request = tonic::Request::new(cluster_manager::DeleteClusterDescriptorRequest {
                cluster_id: Some(cluster_id.into()),
            });

            let response = self.inner.delete_cluster_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                cluster_manager::delete_cluster_descriptor_response::Reply::Failure(failure) => {
                    let error = DeleteClusterDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                cluster_manager::delete_cluster_descriptor_response::Reply::Success(success) => {
                    let cluster_id = extract!(success.cluster_descriptor)?;
                    Ok(cluster_id)
                }
            }
        }

        pub async fn get_cluster_descriptor(&mut self, cluster_id: ClusterId) -> Result<ClusterDescriptor, GetClusterDescriptorError> {
            let request = tonic::Request::new(cluster_manager::GetClusterDescriptorRequest {
                id: Some(cluster_id.into()),
            });

            match self.inner.get_cluster_descriptor(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(GetClusterDescriptorError { cluster_id, message: String::from("Response contains no result!") })?;
                    match result {
                        cluster_manager::get_cluster_descriptor_response::Result::Failure(_) => {
                            Err(GetClusterDescriptorError { cluster_id, message: String::from("Failed to get cluster descriptor!") })
                        }
                        cluster_manager::get_cluster_descriptor_response::Result::Success(cluster_manager::GetClusterDescriptorSuccess { configuration }) => {
                            let configuration = configuration
                                .ok_or(GetClusterDescriptorError { cluster_id, message: String::from("Response contains no cluster descriptor!") })?;
                            ClusterDescriptor::try_from(configuration)
                                .map_err(|_| GetClusterDescriptorError { cluster_id, message: String::from("Conversion failed for cluster descriptors!") })
                        }
                    }
                },
                Err(status) => {
                    Err(GetClusterDescriptorError { cluster_id, message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn list_cluster_descriptors(&mut self) -> Result<Vec<ClusterDescriptor>, ListClusterDescriptorsError> {
            let request = tonic::Request::new(cluster_manager::ListClusterDescriptorsRequest {});

            match self.inner.list_cluster_descriptors(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(ListClusterDescriptorsError { message: String::from("Response contains no result!") })?;
                    match result {
                        cluster_manager::list_cluster_descriptors_response::Result::Failure(_) => {
                            Err(ListClusterDescriptorsError { message: String::from("Failed to list clusters!") })
                        }
                        cluster_manager::list_cluster_descriptors_response::Result::Success(cluster_manager::ListClusterDescriptorsSuccess { configurations }) => {
                            configurations.into_iter()
                                .map(ClusterDescriptor::try_from)
                                .collect::<Result<Vec<ClusterDescriptor>, _>>()
                                .map_err(|_| ListClusterDescriptorsError { message: String::from("Conversion failed for list of cluster descriptors!") })
                        }
                    }
                },
                Err(status) => {
                    Err(ListClusterDescriptorsError { message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn store_cluster_deployment(&mut self, deployment: ClusterDeployment) -> Result<ClusterId, ClientError<StoreClusterDeploymentError>> {

            let request = tonic::Request::new(cluster_manager::StoreClusterDeploymentRequest {
                cluster_deployment: Some(deployment.into()),
            });

            let response = self.inner.store_cluster_deployment(request).await?
                .into_inner();

            match extract!(response.reply)? {
                cluster_manager::store_cluster_deployment_response::Reply::Failure(failure) => {
                    let error = StoreClusterDeploymentError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                cluster_manager::store_cluster_deployment_response::Reply::Success(success) => {
                    let cluster_id = extract!(success.cluster_id)?;
                    Ok(cluster_id)
                }
            }
        }

        pub async fn delete_cluster_deployment(&mut self, cluster_id: ClusterId) -> Result<ClusterDeployment, ClientError<DeleteClusterDeploymentError>> {

            let request = tonic::Request::new(cluster_manager::DeleteClusterDeploymentRequest {
                cluster_id: Some(cluster_id.into()),
            });

            let response = self.inner.delete_cluster_deployment(request).await?
                .into_inner();

            match extract!(response.reply)? {
                cluster_manager::delete_cluster_deployment_response::Reply::Failure(failure) => {
                    let error = DeleteClusterDeploymentError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                cluster_manager::delete_cluster_deployment_response::Reply::Success(success) => {
                    let cluster_id = extract!(success.cluster_deployment)?;
                    Ok(cluster_id)
                }
            }
        }

        pub async fn get_cluster_deployment(&mut self, cluster_id: ClusterId) -> Result<ClusterDeployment, GetClusterDeploymentError> {
            let request = tonic::Request::new(cluster_manager::GetClusterDeploymentRequest {
                id: Some(cluster_id.into()),
            });

            match self.inner.get_cluster_deployment(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(GetClusterDeploymentError::Internal { cluster_id, cause: String::from("Response contains no result!") })?;
                    match result {
                        cluster_manager::get_cluster_deployment_response::Result::Failure(_) => {
                            Err(GetClusterDeploymentError::Internal { cluster_id, cause: String::from("Failed to get cluster deployment!") })
                        }
                        cluster_manager::get_cluster_deployment_response::Result::Success(cluster_manager::GetClusterDeploymentSuccess { deployment }) => {
                            let deployment = deployment
                                .ok_or(GetClusterDeploymentError::Internal { cluster_id, cause: String::from("Response contains no cluster deployment!") })?;
                            ClusterDeployment::try_from(deployment)
                                .map_err(|_| GetClusterDeploymentError::Internal { cluster_id, cause: String::from("Conversion failed for cluster deployment!") })
                        }
                    }
                },
                Err(status) => {
                    Err(GetClusterDeploymentError::Internal { cluster_id, cause: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn list_cluster_deployments(&mut self) -> Result<Vec<ClusterDeployment>, ListClusterDeploymentsError> {
            let request = tonic::Request::new(cluster_manager::ListClusterDeploymentsRequest {});

            match self.inner.list_cluster_deployments(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(ListClusterDeploymentsError { message: String::from("Response contains no result!") })?;
                    match result {
                        cluster_manager::list_cluster_deployments_response::Result::Failure(_) => {
                            Err(ListClusterDeploymentsError { message: String::from("Failed to list clusters!") })
                        }
                        cluster_manager::list_cluster_deployments_response::Result::Success(cluster_manager::ListClusterDeploymentsSuccess { deployments }) => {
                            deployments.into_iter()
                                .map(ClusterDeployment::try_from)
                                .collect::<Result<Vec<ClusterDeployment>, _>>()
                                .map_err(|_| ListClusterDeploymentsError { message: String::from("Conversion failed for list of cluster deployments!") })
                        }
                    }
                },
                Err(status) => {
                    Err(ListClusterDeploymentsError { message: format!("gRPC failure: {status}") })
                }
            }
        }
        
        pub async fn list_cluster_peer_states(&mut self, cluster_id: ClusterId) -> Result<ListClusterPeerStatesResponse, ListClusterPeersError> {
            
            let request = tonic::Request::new(cluster_manager::ListClusterPeerStatesRequest { cluster_id: Some(cluster_id.into()) });
            match self.inner.list_cluster_peer_states(request).await {
                Ok(response) => {
                    let response = response.into_inner();
                    let response: ListClusterPeerStatesResponse = response.try_into()
                        .map_err(|error| ListClusterPeersError { message: format!("Failed to convert response: {error}") } )?;
                    Ok(response)
                }
                Err(status) => {
                    Err(ListClusterPeersError { message: format!("gRPC failure: {status}") })
                }
            }
        }
    }

}
