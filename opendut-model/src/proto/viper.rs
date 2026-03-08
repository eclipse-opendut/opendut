use std::collections::HashMap;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;
use crate::viper::ViperTestParameterKey;

opendut_util::include_proto!("opendut.model.viper");


//
// ViperSource
//

conversion! {
    type Model = crate::viper::ViperSourceId;
    type Proto = ViperSourceId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.uuid.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| Model { uuid: uuid.into() })
    }
}

conversion! {
    type Model = crate::viper::ViperSourceName;
    type Proto = ViperSourceName;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.0
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::viper::ViperSourceDescriptor;
    type Proto = ViperSourceDescriptor;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            url: Some(value.url.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id = extract!(value.id)?
            .try_into()?;

        let name = extract!(value.name)?
            .try_into()?;

        let url = extract!(value.url)?
            .try_into()?;

        Ok(Model { id, name, url })
    }
}


//
// ViperSuite
//

conversion! {
    type Model = crate::viper::ViperTestParameterValueKind;
    type Proto = ViperTestParameterValueKind;

    fn from(value_kind: Model) -> Proto {
        match value_kind {
            crate::viper::ViperTestParameterValueKind::Boolean => ViperTestParameterValueKind::Boolean,
            crate::viper::ViperTestParameterValueKind::Number => ViperTestParameterValueKind::Number,
            crate::viper::ViperTestParameterValueKind::Text => ViperTestParameterValueKind::Text,
        }
    }

    fn try_from(value_kind: Proto) -> ConversionResult<Model> {
        let result = match value_kind {
            ViperTestParameterValueKind::Boolean => crate::viper::ViperTestParameterValueKind::Boolean,
            ViperTestParameterValueKind::Number => crate::viper::ViperTestParameterValueKind::Number,
            ViperTestParameterValueKind::Text => crate::viper::ViperTestParameterValueKind::Text,
        };
        Ok(result)
    }
}

conversion! {
    type Model = crate::viper::ViperTestSuiteDescriptor;
    type Proto = ViperTestSuiteDescriptor;

    fn from(value: Model) -> Proto {
        let Model { id, source, parameters } = value;

        let parameters = parameters.into_iter()
            .map(|(key, value_kind)| ViperTestParameterDescriptor {
                key: key.inner,
                value_kind: i32::from(
                    ViperTestParameterValueKind::from(value_kind)
                ),
            })
            .collect::<Vec<_>>();

        Proto {
            id: Some(id.into()),
            source: Some(source.into()),
            parameters,
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id = extract!(value.id)?
            .try_into()?;

        let source = extract!(value.source)?
            .try_into()?;

        let parameters = value.parameters.into_iter()
            .map(|ViperTestParameterDescriptor { key, value_kind }| {
                let key = crate::viper::ViperTestParameterKey { inner: key };
                let value_kind = crate::viper::ViperTestParameterValueKind::try_from(
                    ViperTestParameterValueKind::try_from(value_kind)
                        .map_err(|cause| ErrorBuilder::message(format!("Unknown enum variant: {cause}")))?
                )?;
                Ok((key, value_kind))
            })
            .collect::<ConversionResult<HashMap<_, _>>>()?;

        Ok(Model { id, source, parameters })
    }
}



//
// ViperTest
//

conversion! {
    type Model = crate::viper::ViperTestRunDescriptor;
    type Proto = ViperTestRunDescriptor;

    fn from(value: Model) -> Proto {
        let parameters = value.parameters.into_iter()
            .map(|(key, value)| {
                ViperTestParameter {
                    key: key.inner,
                    value: Some(value.into())
                }
            })
            .collect::<Vec<_>>();

        Proto {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            source: Some(value.source.into()),
            suite: Some(value.suite.into()),
            cluster: Some(value.cluster.into()),
            parameters,
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id = extract!(value.id)?
            .try_into()?;

        let name = extract!(value.name)?
            .try_into()?;

        let source = extract!(value.source)?
            .try_into()?;

        let suite = extract!(value.suite)?
            .try_into()?;

        let cluster = extract!(value.cluster)?
            .try_into()?;

        let parameters = value.parameters.into_iter()
            .map(|parameter| {
                let key = ViperTestParameterKey { inner: parameter.key };
                let value = extract!(parameter.value)?.try_into()?;

                Ok((key, value))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Model { id, name, source, suite, cluster, parameters })
    }
}

conversion! {
    type Model = crate::viper::ViperTestId;
    type Proto = ViperTestId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.uuid.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| Model { uuid: uuid.into() })
    }
}

conversion! {
    type Model = crate::viper::ViperTestName;
    type Proto = ViperTestName;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.0
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::viper::ViperTestParameterValue;
    type Proto = ViperTestParameterValue;

    fn from(value: Model) -> Proto {
        let value = match value {
            Model::Boolean(value) => viper_test_parameter_value::Kind::Boolean(value),
            Model::Number(value) => viper_test_parameter_value::Kind::Number(value),
            Model::Text(value) => viper_test_parameter_value::Kind::Text(value),
        };
        Proto { kind: Some(value) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let value = match extract!(value.kind)? {
            viper_test_parameter_value::Kind::Boolean(value) => Model::Boolean(value),
            viper_test_parameter_value::Kind::Number(value) => Model::Number(value),
            viper_test_parameter_value::Kind::Text(value) => Model::Text(value),
        };
        Ok(value)
    }
}


//
// ViperRunDeployment
//

conversion! {
    type Model = crate::viper::ViperRunDeployment;
    type Proto = ViperRunDeployment;

    fn from(value: Model) -> Proto {
        Proto {
            run_id: Some(value.run_id.into()),
            test_id: Some(value.test_id.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let run_id = extract!(value.run_id)?
            .try_into()?;

        let test_id = extract!(value.test_id)?
            .try_into()?;

        Ok(Model { run_id, test_id })
    }
}

conversion! {
    type Model = crate::viper::ViperRunId;
    type Proto = ViperRunId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.uuid.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| Model { uuid: uuid.into() })
    }
}
