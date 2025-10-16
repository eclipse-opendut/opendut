use opendut_model::conversion;
use opendut_model::proto::ConversionResult;

tonic::include_proto!("opendut.carl.services.test_manager");


conversion! {
    type Model = crate::carl::test::StoreTestSuiteSourceDescriptorError;
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
