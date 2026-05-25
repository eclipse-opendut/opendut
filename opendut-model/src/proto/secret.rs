use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

opendut_util::include_proto!("opendut.model.secret");


conversion! {
    type Model = crate::secret::SecretId;
    type Proto = SecretId;

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
    type Model = crate::secret::SecretName;
    type Proto = SecretName;

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
    type Model = crate::secret::SecretValue;
    type Proto = SecretValue;

    fn from(value: Model) -> Proto {
        let kind = match value {
            Model::Token(token) => secret_value::Kind::Token(token),
        };
        Proto { kind: Some(kind) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let kind = extract!(value.kind)?;
        let model = match kind {
            secret_value::Kind::Token(token) => Model::Token(token),
        };
        Ok(model)
    }
}

conversion! {
    type Model = crate::secret::SecretDescriptor;
    type Proto = SecretDescriptor;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            value: Some(value.value.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id = extract!(value.id)?
            .try_into()?;

        let name = extract!(value.name)?
            .try_into()?;

        let value = extract!(value.value)?
            .try_into()?;

        Ok(Model { id, name, value })
    }
}
