use std::collections::HashMap;
use opendut_util::conversion;
use opendut_util::proto::ConversionResult;
use crate::viper::ViperRunParameterKey;

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
// ViperRun
//

conversion! {
    type Model = crate::viper::ViperRunDescriptor;
    type Proto = ViperRunDescriptor;

    fn from(value: Model) -> Proto {
        let parameters = value.parameters.into_iter()
            .map(|(key, value)| {
                ViperRunParameter {
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
                let key = ViperRunParameterKey { inner: parameter.key };
                let value = extract!(parameter.value)?.try_into()?;

                Ok((key, value))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Model { id, name, source, suite, cluster, parameters })
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

conversion! {
    type Model = crate::viper::ViperRunName;
    type Proto = ViperRunName;

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
    type Model = crate::viper::ViperRunParameterValue;
    type Proto = ViperRunParameterValue;

    fn from(value: Model) -> Proto {
        let value = match value {
            Model::Boolean(value) => viper_run_parameter_value::Kind::Boolean(value),
            Model::Number(value) => viper_run_parameter_value::Kind::Number(value),
            Model::Text(value) => viper_run_parameter_value::Kind::Text(value),
        };
        Proto { kind: Some(value) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let value = match extract!(value.kind)? {
            viper_run_parameter_value::Kind::Boolean(value) => Model::Boolean(value),
            viper_run_parameter_value::Kind::Number(value) => Model::Number(value),
            viper_run_parameter_value::Kind::Text(value) => Model::Text(value),
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
            id: Some(value.id.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id = extract!(value.id)?
            .try_into()?;

        Ok(Model { id })
    }
}
