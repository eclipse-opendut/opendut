use crate::conversion;
use crate::proto::ConversionResult;

crate::include_proto!("opendut.model.test_suite");


conversion! {
    type Model = crate::test_suite::TestSuiteSourceId;
    type Proto = TestSuiteSourceId;

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
    type Model = crate::test_suite::TestSuiteSourceName;
    type Proto = TestSuiteSourceName;

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
    type Model = crate::test_suite::TestSuiteSourceDescriptor;
    type Proto = TestSuiteSourceDescriptor;

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
