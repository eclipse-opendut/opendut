use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.cleo.rs"));

impl From<crate::cleo::CleoId> for CleoId {
    fn from(value: crate::cleo::CleoId) -> Self {
        Self {
            uuid: Some(value.0.into())
        }
    }
}

impl TryFrom<CleoId> for crate::cleo::CleoId {
    type Error = ConversionError;

    fn try_from(value: CleoId) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<CleoId, crate::cleo::CleoId>;

        value.uuid
            .ok_or(ErrorBuilder::field_not_set("uuid"))
            .map(|uuid| Self(uuid.into()))
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

impl From<crate::cleo::CleoSetup> for CleoSetup {
    fn from(value: crate::cleo::CleoSetup) -> Self {
        Self {
            id: Some(value.id.into()),
            carl: Some(value.carl.into()),
            ca: Some(value.ca.into()),
            auth_config: Some(value.auth_config.into()),
        }
    }
}

impl TryFrom<CleoSetup> for crate::cleo::CleoSetup {
    type Error = ConversionError;

    fn try_from(value: CleoSetup) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<CleoSetup, crate::cleo::CleoSetup>;

        let id: crate::cleo::CleoId = value.id
            .ok_or(ErrorBuilder::field_not_set("id"))?
            .try_into()?;

        let carl: url::Url = value.carl
            .ok_or(ErrorBuilder::field_not_set("carl"))
            .and_then(|url| url::Url::parse(&url.value)
                .map_err(|cause| ErrorBuilder::message(format!("Carl URL could not be parsed: {}", cause))))?;

        let ca: crate::util::net::Certificate = value.ca
            .ok_or(ErrorBuilder::field_not_set("ca"))
            .and_then(crate::util::net::Certificate::try_from)?;

        let auth_config = value.auth_config
            .ok_or(ErrorBuilder::field_not_set("auth_config"))?
            .try_into()?;

        Ok(Self {
            id,
            carl,
            ca,
            auth_config,
        })
    }
}