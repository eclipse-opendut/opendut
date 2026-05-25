use tonic::{Request, Response, Status};
use tracing::{error, trace};

use opendut_carl_api::proto::services::secret_manager::{
    store_secret_response, delete_secret_response, list_secrets_response,
    StoreSecretRequest, StoreSecretResponse, StoreSecretSuccess,
    DeleteSecretRequest, DeleteSecretResponse, DeleteSecretSuccess,
    ListSecretsRequest, ListSecretsResponse, ListSecretsSuccess,
};
use opendut_carl_api::proto::services::secret_manager::secret_manager_server::{SecretManager as SecretManagerService, SecretManagerServer};
use opendut_model::secret::{SecretDescriptor, SecretId};

use crate::manager::grpc::error::LogApiErr;
use crate::manager::grpc::extract;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::PersistenceError;

pub struct SecretManagerFacade {
    pub resource_manager: ResourceManagerRef,
}

impl SecretManagerFacade {
    pub fn into_grpc_service(self) -> super::web::CorsGrpcWeb<SecretManagerServer<Self>> {
        super::web::enable(SecretManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl SecretManagerService for SecretManagerFacade {

    #[tracing::instrument(skip_all, level="trace")]
    async fn store_secret(&self, request: Request<StoreSecretRequest>) -> Result<Response<StoreSecretResponse>, Status> {

        let request = request.into_inner();
        let secret: SecretDescriptor = extract!(request.secret_descriptor)?;

        trace!("Received request to store secret: {secret:?}");

        let result =
            self.resource_manager.insert(secret.id, secret.clone()).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::secret::StoreSecretError::Internal {
                    secret_id: secret.id,
                    secret_name: secret.name,
                    cause: String::from("Error when accessing persistence while storing secret"),
                });

        let reply = match result {
            Ok(()) => store_secret_response::Reply::Success(
                StoreSecretSuccess {
                    secret_id: Some(secret.id.into()),
                }
            ),
            Err(error) => store_secret_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(StoreSecretResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_secret(&self, request: Request<DeleteSecretRequest>) -> Result<Response<DeleteSecretResponse>, Status> {

        let request = request.into_inner();
        let secret_id: SecretId = extract!(request.secret_id)?;

        trace!("Received request to delete secret <{secret_id}>.");

        // Check referential integrity: ensure no ViperSourceDescriptor references this secret
        #[cfg(feature = "viper")]
        {
            use opendut_model::viper::ViperSourceDescriptor;

            let viper_sources = self.resource_manager.list::<ViperSourceDescriptor>().await
                .inspect_err(|error| error!("Error while listing VIPER source descriptors for referential integrity check: {error}"))
                .map_err(|_: PersistenceError| opendut_carl_api::carl::secret::DeleteSecretError::Internal {
                    secret_id,
                    cause: String::from("Error when accessing persistence while checking referential integrity"),
                });

            match viper_sources {
                Ok(sources) => {
                    let referencing_sources: Vec<_> = sources.values()
                        .filter(|source| source.secret_id == Some(secret_id))
                        .collect();

                    if !referencing_sources.is_empty() {
                        let source_names: Vec<String> = referencing_sources.iter()
                            .map(|s| format!("{}", s.name))
                            .collect();
                        let cause = format!(
                            "Secret is still referenced by VIPER source(s): {}",
                            source_names.join(", ")
                        );

                        let reply = delete_secret_response::Reply::Failure(
                            opendut_carl_api::carl::secret::DeleteSecretError::Conflict {
                                secret_id,
                                cause,
                            }.into()
                        );

                        return Ok(Response::new(DeleteSecretResponse {
                            reply: Some(reply),
                        }));
                    }
                }
                Err(error) => {
                    let reply = delete_secret_response::Reply::Failure(error.into());
                    return Ok(Response::new(DeleteSecretResponse {
                        reply: Some(reply),
                    }));
                }
            }
        }

        // Remove the secret
        let result =
            self.resource_manager.remove::<SecretDescriptor>(secret_id).await
                .log_api_err()
                .map_err(|_: PersistenceError| opendut_carl_api::carl::secret::DeleteSecretError::Internal {
                    secret_id,
                    cause: String::from("Error when accessing persistence while deleting secret"),
                });

        let reply = match result {
            Ok(Some(secret_descriptor)) => delete_secret_response::Reply::Success(
                DeleteSecretSuccess {
                    secret_descriptor: Some(secret_descriptor.into()),
                }
            ),
            Ok(None) => delete_secret_response::Reply::Failure(
                opendut_carl_api::carl::secret::DeleteSecretError::NotFound {
                    secret_id,
                }.into()
            ),
            Err(error) => delete_secret_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeleteSecretResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_secrets(&self, _: Request<ListSecretsRequest>) -> Result<Response<ListSecretsResponse>, Status> {

        trace!("Received request to list secrets.");

        let result = self.resource_manager.list::<SecretDescriptor>().await
            .inspect_err(|error| error!("Error while listing secrets from gRPC API: {error}"))
            .map_err(|_: PersistenceError| opendut_carl_api::carl::secret::ListSecretsError::Internal {
                cause: String::from("Error when accessing persistence while listing secrets"),
            });

        let reply = match result {
            Ok(secrets) => {
                let secrets = secrets.into_values()
                    .map(From::from)
                    .collect::<Vec<_>>();

                list_secrets_response::Reply::Success(
                    ListSecretsSuccess { secrets }
                )
            }
            Err(error) => list_secrets_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(ListSecretsResponse {
            reply: Some(reply),
        }))
    }
}
