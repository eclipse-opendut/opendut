use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

tonic::include_proto!("opendut.carl.services.secret_manager");


conversion! {
    type Model = crate::carl::secret::StoreSecretError;
    type Proto = StoreSecretFailure;

    fn from(value: Model) -> Proto {
        let error = match value {
            Model::Internal { secret_id, secret_name, cause } => {
                store_secret_failure::Error::Internal(
                    StoreSecretFailureInternal {
                        secret_id: Some(secret_id.into()),
                        secret_name: Some(secret_name.into()),
                        cause,
                    }
                )
            }
        };

        Proto { error: Some(error) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = match extract!(value.error)? {
            store_secret_failure::Error::Internal(
                StoreSecretFailureInternal { secret_id, secret_name, cause }
            ) => {
                Model::Internal {
                    secret_id: extract!(secret_id)?.try_into()?,
                    secret_name: extract!(secret_name)?.try_into()?,
                    cause,
                }
            }
        };

        Ok(error)
    }
}

conversion! {
    type Model = crate::carl::secret::DeleteSecretError;
    type Proto = DeleteSecretFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::NotFound { secret_id } => {
                delete_secret_failure::Error::NotFound(DeleteSecretFailureNotFound {
                    secret_id: Some(secret_id.into()),
                })
            }
            Model::Conflict { secret_id, cause } => {
                delete_secret_failure::Error::Conflict(DeleteSecretFailureConflict {
                    secret_id: Some(secret_id.into()),
                    cause,
                })
            }
            Model::Internal { secret_id, cause } => {
                delete_secret_failure::Error::Internal(DeleteSecretFailureInternal {
                    secret_id: Some(secret_id.into()),
                    cause,
                })
            }
        };
        Proto {
            error: Some(proto_error)
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = extract!(value.error)?;

        match error {
            delete_secret_failure::Error::NotFound(error) => {
                let secret_id = extract!(error.secret_id)?.try_into()?;
                Ok(Model::NotFound { secret_id })
            }
            delete_secret_failure::Error::Conflict(error) => {
                let secret_id = extract!(error.secret_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Conflict { secret_id, cause })
            }
            delete_secret_failure::Error::Internal(error) => {
                let secret_id = extract!(error.secret_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Internal { secret_id, cause })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::secret::ListSecretsError;
    type Proto = ListSecretsFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::Internal { cause } => {
                list_secrets_failure::Error::Internal(ListSecretsFailureInternal {
                    cause,
                })
            }
        };
        Proto {
            error: Some(proto_error)
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = extract!(value.error)?;

        match error {
            list_secrets_failure::Error::Internal(error) => {
                let cause = error.cause;
                Ok(Model::Internal { cause })
            }
        }
    }
}
