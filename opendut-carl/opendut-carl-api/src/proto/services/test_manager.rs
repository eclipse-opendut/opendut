use opendut_model::viper::TestSuiteSourceName;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

tonic::include_proto!("opendut.carl.services.test_manager");


//
// TestSuiteSourceDescriptor
//

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


//
// TestSuiteRunDescriptor
//

conversion! {
    type Model = crate::carl::viper::StoreTestSuiteRunDescriptorError;
    type Proto = StoreTestSuiteRunDescriptorFailure;

    fn from(value: Model) -> Proto {
        let error = match value {
            Model::Internal { run_id, cause } => {
                store_test_suite_run_descriptor_failure::Error::Internal(
                    StoreTestSuiteRunDescriptorFailureInternal {
                        run_id: Some(run_id.into()),
                        cause,
                    }
                )
            }
        };

        Proto { error: Some(error) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = match extract!(value.error)? {
            store_test_suite_run_descriptor_failure::Error::Internal(
                StoreTestSuiteRunDescriptorFailureInternal { run_id, cause }
            ) => {
                Model::Internal {
                    run_id: extract!(run_id)?.try_into()?,
                    cause,
                }
            }
        };

        Ok(error)
    }
}

conversion! {
    type Model = crate::carl::viper::DeleteTestSuiteRunDescriptorError;
    type Proto = DeleteTestSuiteRunDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::RunNotFound { run_id } => {
                delete_test_suite_run_descriptor_failure::Error::RunNotFound(DeleteTestSuiteRunDescriptorFailureRunNotFound {
                    run_id: Some(run_id.into())
                })
            }
            Model::Internal { run_id, cause } => {
                delete_test_suite_run_descriptor_failure::Error::Internal(DeleteTestSuiteRunDescriptorFailureInternal {
                    run_id: Some(run_id.into()),
                    cause
                })
            }
            Model::ClusterDeploymentExists { run_id, cluster_id } => {
                delete_test_suite_run_descriptor_failure::Error::DeploymentExists(DeleteTestSuiteRunDescriptorFailureDeploymentExists {
                    run_id: Some(run_id.into()),
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
            delete_test_suite_run_descriptor_failure::Error::RunNotFound(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                Ok(Model::RunNotFound { run_id })
            }
            delete_test_suite_run_descriptor_failure::Error::Internal(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    run_id,
                    cause,
                })
            }
            delete_test_suite_run_descriptor_failure::Error::DeploymentExists(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                let cluster_id = extract!(error.cluster_id)?.try_into()?;

                Ok(Model::ClusterDeploymentExists {
                    run_id,
                    cluster_id,
                })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::viper::GetTestSuiteRunDescriptorError;
    type Proto = GetTestSuiteRunDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::RunNotFound { run_id } => {
                get_test_suite_run_descriptor_failure::Error::RunNotFound(GetTestSuiteRunDescriptorFailureRunNotFound {
                    run_id: Some(run_id.into())
                })
            }
            Model::Internal { run_id, cause } => {
                get_test_suite_run_descriptor_failure::Error::Internal(GetTestSuiteRunDescriptorFailureInternal {
                    run_id: Some(run_id.into()),
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
            get_test_suite_run_descriptor_failure::Error::RunNotFound(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                Ok(Model::RunNotFound { run_id })
            }
            get_test_suite_run_descriptor_failure::Error::Internal(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    run_id,
                    cause,
                })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::viper::ListTestSuiteRunDescriptorsError;
    type Proto = ListTestSuiteRunDescriptorsFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::Internal { cause } => {
                list_test_suite_run_descriptors_failure::Error::Internal(ListTestSuiteRunDescriptorsFailureInternal {
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
            list_test_suite_run_descriptors_failure::Error::Internal(error) => {
                let cause = error.cause;
                Ok(Model::Internal {
                    cause,
                })
            }
        }
    }
}


//
// TestSuiteRunDeployment
//

conversion! {
    type Model = crate::carl::viper::StoreTestSuiteRunDeploymentError;
    type Proto = StoreTestSuiteRunDeploymentFailure;

    fn from(value: Model) -> Proto {
        let error = match value {
            Model::Internal { run_id, cause } => {
                store_test_suite_run_deployment_failure::Error::Internal(
                    StoreTestSuiteRunDeploymentFailureInternal {
                        run_id: Some(run_id.into()),
                        cause,
                    }
                )
            }
        };

        Proto { error: Some(error) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = match extract!(value.error)? {
            store_test_suite_run_deployment_failure::Error::Internal(
                StoreTestSuiteRunDeploymentFailureInternal { run_id, cause }
            ) => {
                Model::Internal {
                    run_id: extract!(run_id)?.try_into()?,
                    cause,
                }
            }
        };

        Ok(error)
    }
}

conversion! {
    type Model = crate::carl::viper::DeleteTestSuiteRunDeploymentError;
    type Proto = DeleteTestSuiteRunDeploymentFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::RunDeploymentNotFound { run_id } => {
                delete_test_suite_run_deployment_failure::Error::RunNotFound(DeleteTestSuiteRunDeploymentFailureRunNotFound {
                    run_id: Some(run_id.into())
                })
            }
            Model::Internal { run_id, cause } => {
                delete_test_suite_run_deployment_failure::Error::Internal(DeleteTestSuiteRunDeploymentFailureInternal {
                    run_id: Some(run_id.into()),
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
            delete_test_suite_run_deployment_failure::Error::RunNotFound(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                Ok(Model::RunDeploymentNotFound { run_id })
            }
            delete_test_suite_run_deployment_failure::Error::Internal(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    run_id,
                    cause,
                })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::viper::GetTestSuiteRunDeploymentError;
    type Proto = GetTestSuiteRunDeploymentFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::RunDeploymentNotFound { run_id } => {
                get_test_suite_run_deployment_failure::Error::RunNotFound(GetTestSuiteRunDeploymentFailureRunNotFound {
                    run_id: Some(run_id.into())
                })
            }
            Model::Internal { run_id, cause } => {
                get_test_suite_run_deployment_failure::Error::Internal(GetTestSuiteRunDeploymentFailureInternal {
                    run_id: Some(run_id.into()),
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
            get_test_suite_run_deployment_failure::Error::RunNotFound(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                Ok(Model::RunDeploymentNotFound { run_id })
            }
            get_test_suite_run_deployment_failure::Error::Internal(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    run_id,
                    cause,
                })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::viper::ListTestSuiteRunDeploymentsError;
    type Proto = ListTestSuiteRunDeploymentsFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::Internal { cause } => {
                list_test_suite_run_deployments_failure::Error::Internal(ListTestSuiteRunDeploymentsFailureInternal {
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
            list_test_suite_run_deployments_failure::Error::Internal(error) => {
                let cause = error.cause;
                Ok(Model::Internal {
                    cause,
                })
            }
        }
    }
}
