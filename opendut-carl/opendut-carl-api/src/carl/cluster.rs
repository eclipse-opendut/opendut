#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_types::cluster::{ClusterId, ClusterName};
use opendut_types::cluster::state::{ClusterState, ClusterStates};

#[derive(thiserror::Error, Debug)]
pub enum CreateClusterConfigurationError {
    #[error("ClusterConfigration '{actual_name}' <{actual_id}> could not be created, because ClusterConfigration '{other_name}' <{other_id}> is already registered with the same ClusterId!")]
    ClusterConfigurationAlreadyExists {
        actual_id: ClusterId,
        actual_name: ClusterName,
        other_id: ClusterId,
        other_name: ClusterName
    },
    #[error("ClusterConfigration '{cluster_name}' <{cluster_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterConfigurationError {
    #[error("ClusterConfiguration <{cluster_id}> could not be deleted, because a ClusterConfiguration with that id does not exist!")]
    ClusterConfigurationNotFound {
        cluster_id: ClusterId
    },
    #[error("ClusterConfiguration '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{actual_state}'! A ClusterConfiguration can be deleted when cluster is in state: {required_states}")]
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: ClusterStates,
    },
    #[error("ClusterConfiguration '{cluster_name}' <{cluster_id}> deleted with internal errors:\n  {cause}")]
    Internal {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct GetClusterConfigurationError {
    message: String,
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ListClusterConfigurationsError {
    message: String,
}


#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct StoreClusterDeploymentError {
    message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterDeploymentError {
    #[error("Invalid ClusterId: {cause}")]
    InvalidClusterId {
        cause: String
    },
    #[error("Unknown cluster id <{id}>")]
    ClusterNotFound {
        id: ClusterId
    },
    #[error("Internal error when deleting cluster with id <{id}>.\n{cause}")]
    Internal {
        id: ClusterId,
        cause: String,
    },
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ListClusterDeploymentsError {
    message: String,
}


#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use tonic::codegen::{Body, Bytes, StdError};

    use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};

    use crate::carl::{ClientError, extract};
    use crate::proto::services::cluster_manager;
    use crate::proto::services::cluster_manager::cluster_manager_client::ClusterManagerClient;

    use super::*;

    #[derive(Clone, Debug)]
    pub struct ClusterManager<T> {
        inner: ClusterManagerClient<T>,
    }

    impl<T> ClusterManager<T>
    where T: tonic::client::GrpcService<tonic::body::BoxBody>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: ClusterManagerClient<T>) -> ClusterManager<T> {
            ClusterManager {
                inner
            }
        }

        pub async fn store_cluster_configuration(&mut self, configuration: ClusterConfiguration) -> Result<ClusterId, ClientError<CreateClusterConfigurationError>> {

            let request = tonic::Request::new(cluster_manager::CreateClusterConfigurationRequest {
                cluster_configuration: Some(configuration.into()),
            });

            let response = self.inner.create_cluster_configuration(request).await?
                .into_inner();

            match extract!(response.reply)? {
                cluster_manager::create_cluster_configuration_response::Reply::Failure(failure) => {
                    let error = CreateClusterConfigurationError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                cluster_manager::create_cluster_configuration_response::Reply::Success(success) => {
                    let peer_id = extract!(success.cluster_id)?;
                    Ok(peer_id)
                }
            }
        }

        pub async fn delete_cluster_configuration(&mut self, cluster_id: ClusterId) -> Result<ClusterConfiguration, ClientError<DeleteClusterConfigurationError>> {

            let request = tonic::Request::new(cluster_manager::DeleteClusterConfigurationRequest {
                cluster_id: Some(cluster_id.into()),
            });

            let response = self.inner.delete_cluster_configuration(request).await?
                .into_inner();

            match extract!(response.reply)? {
                cluster_manager::delete_cluster_configuration_response::Reply::Failure(failure) => {
                    let error = DeleteClusterConfigurationError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                cluster_manager::delete_cluster_configuration_response::Reply::Success(success) => {
                    let peer_id = extract!(success.cluster_configuration)?;
                    Ok(peer_id)
                }
            }
        }

        pub async fn get_cluster_configuration(&mut self, id: ClusterId) -> Result<ClusterConfiguration, GetClusterConfigurationError> {
            let request = tonic::Request::new(cluster_manager::GetClusterConfigurationRequest {
                id: Some(id.into()),
            });

            match self.inner.get_cluster_configuration(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(GetClusterConfigurationError { message: format!("Response contains no result!") })?;
                    match result {
                        cluster_manager::get_cluster_configuration_response::Result::Failure(_) => {
                            Err(GetClusterConfigurationError { message: format!("Failed to get cluster configuration!") })
                        }
                        cluster_manager::get_cluster_configuration_response::Result::Success(cluster_manager::GetClusterConfigurationSuccess { configuration }) => {
                            let configuration = configuration
                                .ok_or(GetClusterConfigurationError { message: format!("Response contains no cluster configuration!") })?;
                            ClusterConfiguration::try_from(configuration)
                                .map_err(|_| GetClusterConfigurationError { message: format!("Conversion failed for cluster configurations!") })
                        }
                    }
                },
                Err(status) => {
                    Err(GetClusterConfigurationError { message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn list_cluster_configurations(&mut self) -> Result<Vec<ClusterConfiguration>, ListClusterConfigurationsError> {
            let request = tonic::Request::new(cluster_manager::ListClusterConfigurationsRequest {});

            match self.inner.list_cluster_configurations(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(ListClusterConfigurationsError { message: format!("Response contains no result!") })?;
                    match result {
                        cluster_manager::list_cluster_configurations_response::Result::Failure(_) => {
                            Err(ListClusterConfigurationsError { message: format!("Failed to list clusters!") })
                        }
                        cluster_manager::list_cluster_configurations_response::Result::Success(cluster_manager::ListClusterConfigurationsSuccess { configurations }) => {
                            configurations.into_iter()
                                .map(ClusterConfiguration::try_from)
                                .collect::<Result<Vec<ClusterConfiguration>, _>>()
                                .map_err(|_| ListClusterConfigurationsError { message: format!("Conversion failed for list of cluster configurations!") })
                        }
                    }
                },
                Err(status) => {
                    Err(ListClusterConfigurationsError { message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn store_cluster_deployment(&mut self, deployment: &ClusterDeployment) -> Result<(), StoreClusterDeploymentError> {
            let request = tonic::Request::new(cluster_manager::StoreClusterDeploymentRequest {
                deployment: Some(Clone::clone(deployment).into()),
            });

            match self.inner.store_cluster_deployment(request).await {
                Ok(_) => {
                    Ok(())
                },
                Err(status) => {
                    Err(StoreClusterDeploymentError { message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn delete_cluster_deployment(&mut self, cluster_id: ClusterId) -> Result<ClusterDeployment, ClientError<DeleteClusterDeploymentError>> {

            let request = tonic::Request::new(cluster_manager::DeleteClusterDeploymentRequest {
                id: Some(cluster_id.into()),
            });

            let response = self.inner.delete_cluster_deployment(request)
                .await?
                .into_inner();

            let result = extract!(response.result)?;

            match result {
                cluster_manager::delete_cluster_deployment_response::Result::Failure(failure) => {
                    match failure.reason {
                        Some(cluster_manager::delete_cluster_deployment_failure::Reason::ClusterNotFound(cluster_manager::DeleteClusterDeploymentFailureNotFound { .. })) => {
                            Err(ClientError::UsageError(DeleteClusterDeploymentError::ClusterNotFound { id: cluster_id } ))
                        }
                        Some(cluster_manager::delete_cluster_deployment_failure::Reason::InvalidClusterId(cluster_manager::DeleteClusterDeploymentFailureInvalidClusterId { cause })) => {
                            Err(ClientError::UsageError(DeleteClusterDeploymentError::InvalidClusterId { cause } ))
                        }
                        Some(cluster_manager::delete_cluster_deployment_failure::Reason::ClusterIdRequired(_)) => {
                            Err(ClientError::InvalidRequest(format!("DeleteClusterDeploymentRequest requires ClusterId!")))
                        }
                        None => {
                            Err(ClientError::InvalidRequest(format!("DeleteClusterDeploymentFailure contains no reason!")))
                        }
                        Some(cluster_manager::delete_cluster_deployment_failure::Reason::Internal(cluster_manager::DeleteClusterDeploymentFailureInternal { cause, .. })) => {
                            Err(ClientError::UsageError(DeleteClusterDeploymentError::Internal { id: cluster_id, cause } ))
                        }
                    }
                }
                cluster_manager::delete_cluster_deployment_response::Result::Success(success) => {
                    let cluster_deployment = extract!(success.deployment)?;
                    Ok(cluster_deployment)
                }
            }
        }

        pub async fn list_cluster_deployments(&mut self) -> Result<Vec<ClusterDeployment>, ListClusterDeploymentsError> {
            let request = tonic::Request::new(cluster_manager::ListClusterDeploymentsRequest {});

            match self.inner.list_cluster_deployments(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(ListClusterDeploymentsError { message: format!("Response contains no result!") })?;
                    match result {
                        cluster_manager::list_cluster_deployments_response::Result::Failure(_) => {
                            Err(ListClusterDeploymentsError { message: format!("Failed to list clusters!") })
                        }
                        cluster_manager::list_cluster_deployments_response::Result::Success(cluster_manager::ListClusterDeploymentsSuccess { deployments }) => {
                            deployments.into_iter()
                                .map(ClusterDeployment::try_from)
                                .collect::<Result<Vec<ClusterDeployment>, _>>()
                                .map_err(|_| ListClusterDeploymentsError { message: format!("Conversion failed for list of cluster deployments!") })
                        }
                    }
                },
                Err(status) => {
                    Err(ListClusterDeploymentsError { message: format!("gRPC failure: {status}") })
                }
            }
        }
    }
}
