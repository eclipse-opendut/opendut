use opendut_util::conversion;
use opendut_util::proto::ConversionResult;
use crate::runtime::types::naming::Identifier;

opendut_util::include_proto!("opendut.viper.rt.test_suite");


conversion! {
    type Model = crate::runtime::types::naming::TestSuiteIdentifier;
    type Proto = TestSuiteIdentifier;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.as_str().to_owned()
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let result = Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(format!("Error while parsing TestSuiteIdentifier from Protobuf message: {cause}")))?;

        Ok(result)
    }
}
