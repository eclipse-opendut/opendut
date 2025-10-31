use opendut_model::viper::TestSuiteSourceName;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

tonic::include_proto!("opendut.carl.services.test_manager");


conversion! {
    type Model = crate::carl::viper::StoreTestSuiteSourceDescriptorError;
    type Proto = StoreTestSuiteSourceDescriptorFailure;

    fn from(value: Model) -> Proto {
        let error = match value {
            Model::Internal { source_id, source_name, cause } => {
                store_test_suite_source_descriptor_failure::Error::Internal(
                    StoreTestSuiteSourceDescriptorFailureInternal {
                        source_id: Some(source_id.into()),
                        source_name: Some(source_name.into()),
                        cause,
                    }
                )
            }
        };

        Proto { error: Some(error) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = match extract!(value.error)? {
            store_test_suite_source_descriptor_failure::Error::Internal(
                StoreTestSuiteSourceDescriptorFailureInternal { source_id, source_name, cause }
            ) => {
                Model::Internal {
                    source_id: extract!(source_id)?.try_into()?,
                    source_name: extract!(source_name)?.try_into()?,
                    cause,
                }
            }
        };

        Ok(error)
    }
}


conversion! {
    type Model = crate::carl::viper::DeleteTestSuiteSourceDescriptorError;
    type Proto = DeleteTestSuiteSourceDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::SourceNotFound { source_id } => {
                delete_test_suite_source_descriptor_failure::Error::SourceNotFound(DeleteTestSuiteSourceDescriptorFailureSourceNotFound {
                    source_id: Some(source_id.into())
                })
            }
            Model::Internal { source_id, source_name, cause } => {
                delete_test_suite_source_descriptor_failure::Error::Internal(DeleteTestSuiteSourceDescriptorFailureInternal {
                    source_id: Some(source_id.into()),
                    source_name: source_name.map(Into::into),
                    cause
                })
            }
            Model::ClusterDeploymentExists { source_id, cluster_id } => {
                delete_test_suite_source_descriptor_failure::Error::DeploymentExists(DeleteTestSuiteSourceDescriptorFailureDeploymentExists {
                    source_id: Some(source_id.into()),
                    cluster_id: Some(cluster_id.into()),
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
            delete_test_suite_source_descriptor_failure::Error::SourceNotFound(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                Ok(Model::SourceNotFound { source_id })
            }
            delete_test_suite_source_descriptor_failure::Error::Internal(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let source_name: Option<TestSuiteSourceName> = error.source_name
                    .map(TryInto::try_into)
                    .transpose()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    source_id,
                    source_name,
                    cause,
                })
            }
            delete_test_suite_source_descriptor_failure::Error::DeploymentExists(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let cluster_id = extract!(error.cluster_id)?.try_into()?;

                Ok(Model::ClusterDeploymentExists {
                    source_id,
                    cluster_id,
                })
            }
        }
    }
}


conversion! {
    type Model = crate::carl::viper::GetTestSuiteSourceDescriptorError;
    type Proto = GetTestSuiteSourceDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::SourceNotFound { source_id } => {
                get_test_suite_source_descriptor_failure::Error::SourceNotFound(GetTestSuiteSourceDescriptorFailureSourceNotFound {
                    source_id: Some(source_id.into())
                })
            }
            Model::Internal { source_id, cause } => {
                get_test_suite_source_descriptor_failure::Error::Internal(GetTestSuiteSourceDescriptorFailureInternal {
                    source_id: Some(source_id.into()),
                    cause
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
            get_test_suite_source_descriptor_failure::Error::SourceNotFound(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                Ok(Model::SourceNotFound { source_id })
            }
            get_test_suite_source_descriptor_failure::Error::Internal(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    source_id,
                    cause,
                })
            }
        }
    }
}


conversion! {
    type Model = crate::carl::viper::ListTestSuiteSourceDescriptorsError;
    type Proto = ListTestSuiteSourceDescriptorsFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::Internal { cause } => {
                list_test_suite_source_descriptors_failure::Error::Internal(ListTestSuiteSourceDescriptorsFailureInternal {
                    cause
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
            list_test_suite_source_descriptors_failure::Error::Internal(error) => {
                let cause = error.cause;
                Ok(Model::Internal {
                    cause,
                })
            }
        }
    }
}
