use opendut_model::viper::ViperSourceName;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

tonic::include_proto!("opendut.carl.services.test_manager");


//
// ViperSourceDescriptor
//

conversion! {
    type Model = crate::carl::viper::StoreViperSourceDescriptorError;
    type Proto = StoreViperSourceDescriptorFailure;

    fn from(value: Model) -> Proto {
        let error = match value {
            Model::Internal { source_id, source_name, cause } => {
                store_viper_source_descriptor_failure::Error::Internal(
                    StoreViperSourceDescriptorFailureInternal {
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
            store_viper_source_descriptor_failure::Error::Internal(
                StoreViperSourceDescriptorFailureInternal { source_id, source_name, cause }
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
    type Model = crate::carl::viper::DeleteViperSourceDescriptorError;
    type Proto = DeleteViperSourceDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::SourceNotFound { source_id } => {
                delete_viper_source_descriptor_failure::Error::SourceNotFound(DeleteViperSourceDescriptorFailureSourceNotFound {
                    source_id: Some(source_id.into())
                })
            }
            Model::Internal { source_id, source_name, cause } => {
                delete_viper_source_descriptor_failure::Error::Internal(DeleteViperSourceDescriptorFailureInternal {
                    source_id: Some(source_id.into()),
                    source_name: source_name.map(Into::into),
                    cause
                })
            }
            Model::TestExists { source_id, test_id } => {
                delete_viper_source_descriptor_failure::Error::TestExists(DeleteViperSourceDescriptorFailureTestExists {
                    source_id: Some(source_id.into()),
                    test_id: Some(test_id.into()),
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
            delete_viper_source_descriptor_failure::Error::SourceNotFound(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                Ok(Model::SourceNotFound { source_id })
            }
            delete_viper_source_descriptor_failure::Error::Internal(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let source_name: Option<ViperSourceName> = error.source_name
                    .map(TryInto::try_into)
                    .transpose()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    source_id,
                    source_name,
                    cause,
                })
            }
            delete_viper_source_descriptor_failure::Error::TestExists(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let test_id = extract!(error.test_id)?.try_into()?;

                Ok(Model::TestExists {
                    source_id,
                    test_id,
                })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::viper::GetViperSourceDescriptorError;
    type Proto = GetViperSourceDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::SourceNotFound { source_id } => {
                get_viper_source_descriptor_failure::Error::SourceNotFound(GetViperSourceDescriptorFailureSourceNotFound {
                    source_id: Some(source_id.into())
                })
            }
            Model::Internal { source_id, cause } => {
                get_viper_source_descriptor_failure::Error::Internal(GetViperSourceDescriptorFailureInternal {
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
            get_viper_source_descriptor_failure::Error::SourceNotFound(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                Ok(Model::SourceNotFound { source_id })
            }
            get_viper_source_descriptor_failure::Error::Internal(error) => {
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
    type Model = crate::carl::viper::ListViperSourceDescriptorsError;
    type Proto = ListViperSourceDescriptorsFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::Internal { cause } => {
                list_viper_source_descriptors_failure::Error::Internal(ListViperSourceDescriptorsFailureInternal {
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
            list_viper_source_descriptors_failure::Error::Internal(error) => {
                let cause = error.cause;
                Ok(Model::Internal {
                    cause,
                })
            }
        }
    }
}


//
// ViperTestSuiteParameters
//

conversion! {
    type Model = crate::carl::viper::GetViperTestSuiteParametersError;
    type Proto = GetViperTestSuiteParametersFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::SourceNotFound { source_id} => {
                get_viper_test_suite_parameters_failure::Error::SourceNotFound(GetViperTestSuiteParametersFailureSourceNotFound {
                    source_id: Some(source_id.into()),
                })
            }
            Model::Compilation { source_id, source_name } => {
                get_viper_test_suite_parameters_failure::Error::Compilation(GetViperTestSuiteParametersFailureCompilation {
                    source_id: Some(source_id.into()),
                    source_name: Some(source_name.into()),
                })
            }
            Model::ViperRuntime { source_id, source_name } => {
                get_viper_test_suite_parameters_failure::Error::ViperRuntime(GetViperTestSuiteParametersFailureViperRuntime {
                    source_id: Some(source_id.into()),
                    source_name: Some(source_name.into()),
                })
            }
            Model::Internal { source_id, cause } => {
                get_viper_test_suite_parameters_failure::Error::Internal(GetViperTestSuiteParametersFailureInternal {
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
            get_viper_test_suite_parameters_failure::Error::SourceNotFound(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;

                Ok(Model::SourceNotFound { source_id })
            }
            get_viper_test_suite_parameters_failure::Error::Compilation(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let source_name = extract!(error.source_name)?.try_into()?;
                
                Ok(Model::Compilation { source_id, source_name })
            },
            get_viper_test_suite_parameters_failure::Error::ViperRuntime(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let source_name = extract!(error.source_name)?.try_into()?;
                
                Ok(Model::ViperRuntime{ source_id, source_name })
            }
            get_viper_test_suite_parameters_failure::Error::Internal(error) => {
                let source_id = extract!(error.source_id)?.try_into()?;
                let cause = error.cause;

                Ok(Model::Internal { source_id, cause })
            }
        }
    }
}


//
// ViperTestRunDescriptor
//

conversion! {
    type Model = crate::carl::viper::StoreViperTestRunDescriptorError;
    type Proto = StoreViperTestRunDescriptorFailure;

    fn from(value: Model) -> Proto {
        let error = match value {
            Model::Internal { test_id, cause } => {
                store_viper_test_run_descriptor_failure::Error::Internal(
                    StoreViperTestRunDescriptorFailureInternal {
                        test_id: Some(test_id.into()),
                        cause,
                    }
                )
            }
        };

        Proto { error: Some(error) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let error = match extract!(value.error)? {
            store_viper_test_run_descriptor_failure::Error::Internal(
                StoreViperTestRunDescriptorFailureInternal { test_id, cause }
            ) => {
                Model::Internal {
                    test_id: extract!(test_id)?.try_into()?,
                    cause,
                }
            }
        };

        Ok(error)
    }
}

conversion! {
    type Model = crate::carl::viper::DeleteViperTestRunDescriptorError;
    type Proto = DeleteViperTestRunDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::TestNotFound { test_id } => {
                delete_viper_test_run_descriptor_failure::Error::TestNotFound(DeleteViperTestRunDescriptorFailureTestNotFound {
                    test_id: Some(test_id.into())
                })
            }
            Model::Internal { test_id, cause } => {
                delete_viper_test_run_descriptor_failure::Error::Internal(DeleteViperTestRunDescriptorFailureInternal {
                    test_id: Some(test_id.into()),
                    cause
                })
            }
            Model::ViperRunDeploymentExists { test_id, run_id } => {
                delete_viper_test_run_descriptor_failure::Error::ViperRunDeploymentExists(DeleteViperTestDescriptorFailureViperRunDeploymentExists {
                    test_id: Some(test_id.into()),
                    run_id: Some(run_id.into()),
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
            delete_viper_test_run_descriptor_failure::Error::TestNotFound(error) => {
                let test_id = extract!(error.test_id)?.try_into()?;
                Ok(Model::TestNotFound { test_id })
            }
            delete_viper_test_run_descriptor_failure::Error::Internal(error) => {
                let test_id = extract!(error.test_id)?.try_into()?;
                let cause = error.cause;

                Ok(Model::Internal {
                    test_id,
                    cause,
                })
            }
            delete_viper_test_run_descriptor_failure::Error::ViperRunDeploymentExists(error) => {
                let test_id = extract!(error.test_id)?.try_into()?;
                let run_id = extract!(error.run_id)?.try_into()?;

                Ok(Model::ViperRunDeploymentExists {
                    test_id,
                    run_id,
                })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::viper::GetViperTestRunDescriptorError;
    type Proto = GetViperTestRunDescriptorFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::TestNotFound { test_id } => {
                get_viper_test_run_descriptor_failure::Error::TestNotFound(GetViperTestRunDescriptorFailureTestNotFound {
                    test_id: Some(test_id.into())
                })
            }
            Model::Internal { test_id, cause } => {
                get_viper_test_run_descriptor_failure::Error::Internal(GetViperTestRunDescriptorFailureInternal {
                    test_id: Some(test_id.into()),
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
            get_viper_test_run_descriptor_failure::Error::TestNotFound(error) => {
                let test_id = extract!(error.test_id)?.try_into()?;
                Ok(Model::TestNotFound { test_id })
            }
            get_viper_test_run_descriptor_failure::Error::Internal(error) => {
                let test_id = extract!(error.test_id)?.try_into()?;
                let cause = error.cause;
                Ok(Model::Internal {
                    test_id,
                    cause,
                })
            }
        }
    }
}

conversion! {
    type Model = crate::carl::viper::ListViperTestRunDescriptorsError;
    type Proto = ListViperTestRunDescriptorsFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::Internal { cause } => {
                list_viper_test_run_descriptors_failure::Error::Internal(ListViperTestRunDescriptorsFailureInternal {
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
            list_viper_test_run_descriptors_failure::Error::Internal(error) => {
                let cause = error.cause;
                Ok(Model::Internal {
                    cause,
                })
            }
        }
    }
}


//
// ViperRunDeployment
//

conversion! {
    type Model = crate::carl::viper::StoreViperRunDeploymentError;
    type Proto = StoreViperRunDeploymentFailure;

    fn from(value: Model) -> Proto {
        let error = match value {
            Model::Internal { run_id, cause } => {
                store_viper_run_deployment_failure::Error::Internal(
                    StoreViperRunDeploymentFailureInternal {
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
            store_viper_run_deployment_failure::Error::Internal(
                StoreViperRunDeploymentFailureInternal { run_id, cause }
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
    type Model = crate::carl::viper::DeleteViperRunDeploymentError;
    type Proto = DeleteViperRunDeploymentFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::RunDeploymentNotFound { run_id } => {
                delete_viper_run_deployment_failure::Error::RunNotFound(DeleteViperRunDeploymentFailureRunNotFound {
                    run_id: Some(run_id.into())
                })
            }
            Model::Internal { run_id, cause } => {
                delete_viper_run_deployment_failure::Error::Internal(DeleteViperRunDeploymentFailureInternal {
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
            delete_viper_run_deployment_failure::Error::RunNotFound(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                Ok(Model::RunDeploymentNotFound { run_id })
            }
            delete_viper_run_deployment_failure::Error::Internal(error) => {
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
    type Model = crate::carl::viper::GetViperRunDeploymentError;
    type Proto = GetViperRunDeploymentFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::RunDeploymentNotFound { run_id } => {
                get_viper_run_deployment_failure::Error::RunNotFound(GetViperRunDeploymentFailureRunNotFound {
                    run_id: Some(run_id.into())
                })
            }
            Model::Internal { run_id, cause } => {
                get_viper_run_deployment_failure::Error::Internal(GetViperRunDeploymentFailureInternal {
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
            get_viper_run_deployment_failure::Error::RunNotFound(error) => {
                let run_id = extract!(error.run_id)?.try_into()?;
                Ok(Model::RunDeploymentNotFound { run_id })
            }
            get_viper_run_deployment_failure::Error::Internal(error) => {
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
    type Model = crate::carl::viper::ListViperRunDeploymentsError;
    type Proto = ListViperRunDeploymentsFailure;

    fn from(value: Model) -> Proto {
        let proto_error = match value {
            Model::Internal { cause } => {
                list_viper_run_deployments_failure::Error::Internal(ListViperRunDeploymentsFailureInternal {
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
            list_viper_run_deployments_failure::Error::Internal(error) => {
                let cause = error.cause;
                Ok(Model::Internal {
                    cause,
                })
            }
        }
    }
}
