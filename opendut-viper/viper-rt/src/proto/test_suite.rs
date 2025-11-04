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

conversion! {
    type Model = crate::runtime::types::compile::parameters::ParameterDescriptor;
    type Proto = TestSuiteParameterDescriptor;

    fn from(value: Model) -> Proto {
        use test_suite_parameter_descriptor::Kind;

        let kind = match value {
            Model::BooleanParameter { name, info, default } =>
                Kind::Boolean(TestSuiteParameterDescriptorBoolean {
                    name: name.into(),
                    info: Some(info.into()),
                    default,
                }),
            Model::NumberParameter { name, info, default, min, max } =>
                Kind::Number(TestSuiteParameterDescriptorNumber {
                    name: name.into(),
                    info: Some(info.into()),
                    default,
                    min,
                    max,
                }),
            Model::TextParameter { name, info, default, max } =>
                Kind::Text(TestSuiteParameterDescriptorText {
                    name: name.into(),
                    info: Some(info.into()),
                    default,
                    max_length: max,
                }),
        };

        Proto { kind: Some(kind) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        use test_suite_parameter_descriptor::Kind;

        let model = match extract!(value.kind)? {
            Kind::Boolean(TestSuiteParameterDescriptorBoolean { name, info, default }) =>
                Model::BooleanParameter {
                    name: name.try_into()
                        .map_err(|cause| ErrorBuilder::message(format!("Error while converting ParameterName from Protobuf: {cause}")))?,
                    info: extract!(info)?
                        .try_into()?,
                    default,
                },
            Kind::Number(TestSuiteParameterDescriptorNumber { name, info, default, min, max }) =>
                Model::NumberParameter {
                    name: name.try_into()
                        .map_err(|cause| ErrorBuilder::message(format!("Error while converting ParameterName from ProtoBuf: {cause}")))?,
                    info: extract!(info)?
                        .try_into()?,
                    default,
                    min,
                    max,
                },
            Kind::Text(TestSuiteParameterDescriptorText { name, info, default, max_length }) =>
                Model::TextParameter {
                    name: name.try_into()
                        .map_err(|cause| ErrorBuilder::message(format!("Error while converting ParameterName from ProtoBuf: {cause}")))?,
                    info: extract!(info)?
                        .try_into()?,
                    default,
                    max: max_length,
                },
        };

        Ok(model)
    }
}


conversion! {
    type Model = crate::runtime::types::compile::parameters::ParameterInfo;
    type Proto = TestSuiteParameterInfo;

    fn from(value: Model) -> Proto {
        Proto {
            display_name: value.display_name,
            description: value.description,
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Ok(Model {
            display_name: value.display_name,
            description: value.description,
        })
    }
}
