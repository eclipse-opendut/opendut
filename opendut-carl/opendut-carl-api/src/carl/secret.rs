#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_model::secret::{SecretId, SecretName};
use opendut_model::format::format_id_with_name;


#[derive(thiserror::Error, Debug)]
pub enum StoreSecretError {
    #[error("Secret {secret} could not be stored, due to internal errors:\n  {cause}", secret=format_id_with_name(secret_id, secret_name))]
    Internal {
        secret_id: SecretId,
        secret_name: SecretName,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteSecretError {
    #[error("Secret <{secret_id}> could not be deleted, because a secret with that ID does not exist!")]
    NotFound {
        secret_id: SecretId,
    },
    #[error("Secret <{secret_id}> could not be deleted, because it is still referenced:\n  {cause}")]
    Conflict {
        secret_id: SecretId,
        cause: String,
    },
    #[error("Secret <{secret_id}> could not be deleted, due to internal errors:\n  {cause}")]
    Internal {
        secret_id: SecretId,
        cause: String,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListSecretsError {
    #[error("An internal error occurred computing the list of secrets:\n  {cause}")]
    Internal {
        cause: String,
    }
}


#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use super::*;
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};
    use opendut_model::secret::{SecretDescriptor, SecretId};
    use crate::carl::{extract, ClientError};
    use crate::proto::services::secret_manager;
    use crate::proto::services::secret_manager::secret_manager_client::SecretManagerClient;

    #[derive(Debug, Clone)]
    pub struct SecretManager<T> {
        inner: SecretManagerClient<T>,
    }

    impl<T> SecretManager<T>
    where T: tonic::client::GrpcService<tonic::body::Body>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: SecretManagerClient<T>) -> SecretManager<T> {
            SecretManager {
                inner
            }
        }

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> SecretManager<InterceptedService<T, F>>
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
            let inner_client = SecretManagerClient::new(InterceptedService::new(inner, interceptor));
            SecretManager {
                inner: inner_client
            }
        }

        pub async fn store_secret_descriptor(&mut self, descriptor: SecretDescriptor) -> Result<SecretId, ClientError<StoreSecretError>> {

            let request = tonic::Request::new(secret_manager::StoreSecretRequest {
                secret_descriptor: Some(descriptor.into()),
            });

            let response = self.inner.store_secret(request).await?
                .into_inner();

            match extract!(response.reply)? {
                secret_manager::store_secret_response::Reply::Failure(failure) => {
                    let error = StoreSecretError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                secret_manager::store_secret_response::Reply::Success(success) => {
                    let secret_id = extract!(success.secret_id)?;
                    Ok(secret_id)
                }
            }
        }

        pub async fn delete_secret_descriptor(&mut self, secret_id: SecretId) -> Result<SecretDescriptor, ClientError<DeleteSecretError>> {

            let request = tonic::Request::new(secret_manager::DeleteSecretRequest {
                secret_id: Some(secret_id.into()),
            });

            let response = self.inner.delete_secret(request).await?
                .into_inner();

            match extract!(response.reply)? {
                secret_manager::delete_secret_response::Reply::Failure(failure) => {
                    let error = DeleteSecretError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                secret_manager::delete_secret_response::Reply::Success(success) => {
                    let secret_descriptor = extract!(success.secret_descriptor)?;
                    Ok(secret_descriptor)
                }
            }
        }

        pub async fn list_secret_descriptors(&mut self) -> Result<Vec<SecretDescriptor>, ClientError<ListSecretsError>> {

            let request = tonic::Request::new(secret_manager::ListSecretsRequest {});

            let response = self.inner.list_secrets(request).await?
                .into_inner();

            match extract!(response.reply)? {
                secret_manager::list_secrets_response::Reply::Failure(failure) => {
                    let error = ListSecretsError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                secret_manager::list_secrets_response::Reply::Success(success) => {
                    Ok(success.secrets.into_iter()
                        .map(SecretDescriptor::try_from)
                        .collect::<Result<Vec<_>, _>>()?
                    )
                }
            }
        }
    }
}
