use std::fmt::{Display, Formatter};

#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;
use opendut_types::cluster::{ClusterId, ClusterName};
use opendut_types::cluster::state::ClusterState;
use opendut_types::ShortName;

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
    ClusterConfigurationNotFound {
        cluster_id: ClusterId
    },
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    Internal {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        cause: String
    }
}
impl Display for DeleteClusterConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id } => {
                writeln!(f, "ClusterConfiguration <{cluster_id}> could not be deleted, because a ClusterConfiguration with that id does not exist!")
            }
            DeleteClusterConfigurationError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } => {
                let actual_state = actual_state.short_name();
                let required_states = ClusterState::short_names_joined(required_states);
                writeln!(f, "ClusterConfiguration '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{actual_state}'! A ClusterConfiguration can be deleted when cluster is in state: {required_states}")
            }
            DeleteClusterConfigurationError::Internal { cluster_id, cluster_name, cause } => {
                let cluster_name = match cluster_name {
                    Some(cluster_name) => format!("'{cluster_name}' "),
                    None => String::new(),
                };
                writeln!(f, "ClusterConfiguration '{cluster_name}' <{cluster_id}> deleted with internal errors:\n  {cause}")
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("ClusterConfiguration <{cluster_id}> could not be retrieved:\n  {message}")]
pub struct GetClusterConfigurationError {
    pub cluster_id: ClusterId,
    pub message: String
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct ListClusterConfigurationsError {
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum StoreClusterDeploymentError {
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    Internal {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        cause: String
    }
}
impl Display for StoreClusterDeploymentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreClusterDeploymentError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } => {
                let actual_state = actual_state.short_name();
                let required_states = ClusterState::short_names_joined(required_states);
                writeln!(f, "ClusterDeployment for cluster '{cluster_name}' <{cluster_id}> cannot be changed when cluster is in state '{actual_state}'! A cluster can be updated when: {required_states}")
            }
            StoreClusterDeploymentError::Internal { cluster_id, cluster_name, cause } => {
                let cluster_name = match cluster_name {
                    Some(cluster_name) => format!("'{cluster_name}' "),
                    None => String::from(""),
                };
                writeln!(f, "ClusterDeployment for cluster {cluster_name}<{cluster_id}> could not be changed, due to internal errors:\n  {cause}")
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterDeploymentError {
    ClusterDeploymentNotFound {
        cluster_id: ClusterId
    },
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    Internal {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        cause: String
    },
}
impl Display for DeleteClusterDeploymentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id } => {
                writeln!(f, "ClusterDeployment for cluster <{cluster_id}> could not be deleted, because a ClusterDeployment with that id does not exist!")
            }
            DeleteClusterDeploymentError::IllegalClusterState { cluster_id, cluster_name, actual_state, required_states } => {
                writeln!(f, "ClusterDeployment for cluster '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{}'! A peer can be deleted when: {}", actual_state.short_name(), ClusterState::short_names_joined(required_states))
            }
            DeleteClusterDeploymentError::Internal { cluster_id, cluster_name, cause } => {
                let cluster_name = match cluster_name {
                    Some(cluster_name) => format!("'{cluster_name}' "),
                    None => String::from(""),
                };
                writeln!(f, "ClusterDeployment for cluster {cluster_name}<{cluster_id}> deleted with internal errors:\n  {cause}")
            }
        }
    }
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


#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};

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

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> ClusterManager<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::BoxBody>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::BoxBody>,
                >>::Error: Into<StdError> + Send + Sync,
        {
            let inner_client = ClusterManagerClient::new(InterceptedService::new(inner, interceptor));
            ClusterManager {
                inner: inner_client
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
                    let cluster_id = extract!(success.cluster_configuration)?;
                    Ok(cluster_id)
                }
            }
        }

        pub async fn get_cluster_configuration(&mut self, cluster_id: ClusterId) -> Result<ClusterConfiguration, GetClusterConfigurationError> {
            let request = tonic::Request::new(cluster_manager::GetClusterConfigurationRequest {
                id: Some(cluster_id.into()),
            });

            match self.inner.get_cluster_configuration(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(GetClusterConfigurationError { cluster_id, message: String::from("Response contains no result!") })?;
                    match result {
                        cluster_manager::get_cluster_configuration_response::Result::Failure(_) => {
                            Err(GetClusterConfigurationError { cluster_id, message: String::from("Failed to get cluster configuration!") })
                        }
                        cluster_manager::get_cluster_configuration_response::Result::Success(cluster_manager::GetClusterConfigurationSuccess { configuration }) => {
                            let configuration = configuration
                                .ok_or(GetClusterConfigurationError { cluster_id, message: String::from("Response contains no cluster configuration!") })?;
                            ClusterConfiguration::try_from(configuration)
                                .map_err(|_| GetClusterConfigurationError { cluster_id, message: String::from("Conversion failed for cluster configurations!") })
                        }
                    }
                },
                Err(status) => {
                    Err(GetClusterConfigurationError { cluster_id, message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn list_cluster_configurations(&mut self) -> Result<Vec<ClusterConfiguration>, ListClusterConfigurationsError> {
            let request = tonic::Request::new(cluster_manager::ListClusterConfigurationsRequest {});

            match self.inner.list_cluster_configurations(request).await {
                Ok(response) => {
                    let result = response.into_inner().result
                        .ok_or(ListClusterConfigurationsError { message: String::from("Response contains no result!") })?;
                    match result {
                        cluster_manager::list_cluster_configurations_response::Result::Failure(_) => {
                            Err(ListClusterConfigurationsError { message: String::from("Failed to list clusters!") })
                        }
                        cluster_manager::list_cluster_configurations_response::Result::Success(cluster_manager::ListClusterConfigurationsSuccess { configurations }) => {
                            configurations.into_iter()
                                .map(ClusterConfiguration::try_from)
                                .collect::<Result<Vec<ClusterConfiguration>, _>>()
                                .map_err(|_| ListClusterConfigurationsError { message: String::from("Conversion failed for list of cluster configurations!") })
                        }
                    }
                },
                Err(status) => {
                    Err(ListClusterConfigurationsError { message: format!("gRPC failure: {status}") })
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
    }
}
