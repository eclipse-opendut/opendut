use opendut_util::conversion;
use opendut_util::proto::ConversionResult;

opendut_util::include_proto!("opendut.model.cleo");


conversion! {
    type Model = crate::cleo::CleoId;
    type Proto = CleoId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.uuid.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| crate::cleo::CleoId { uuid: uuid.into() })
    }
}

impl From<uuid::Uuid> for CleoId {
    fn from(value: uuid::Uuid) -> Self {
        let (msb, lsb) = value.as_u64_pair();
        Self {
            uuid: Some(crate::proto::util::Uuid { msb, lsb })
        }
    }
}

conversion! {
    type Model = crate::cleo::CleoSetup;
    type Proto = CleoSetup;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            carl: Some(value.carl.into()),
            ca: Some(value.ca.into()),
            auth_config: Some(value.auth_config.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id: crate::cleo::CleoId = extract!(value.id)?.try_into()?;

        let carl: url::Url = extract!(value.carl)
            .and_then(|url| url::Url::parse(&url.value)
                .map_err(|cause| ErrorBuilder::message(format!("Carl URL could not be parsed: {cause}"))))?;

        let ca: crate::util::net::Certificate = extract!(value.ca)
            .and_then(crate::util::net::Certificate::try_from)?;

        let auth_config = extract!(value.auth_config)?.try_into()?;

        Ok(Model {
            id, carl, ca, auth_config,
        })
    }
}
