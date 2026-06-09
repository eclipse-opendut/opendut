pub mod validation;

use url::Url;
use opendut_lea_components::UserInputValue;
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperSourceKind, ViperSourceName};

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ViperSourceMisconfigurationError {
    #[error("Invalid viper source name")]
    InvalidSourceName,
    #[error("Invalid viper source URL")]
    InvalidSourceUrl,
    #[error("Invalid viper source kind")]
    InvalidSourceKind,
}

#[derive(Clone, Debug)]
pub struct UserViperSourceConfiguration {
    pub id: ViperSourceId,
    pub name: UserInputValue,
    pub url: UserInputValue,
    pub kind: UserInputValue,
    pub is_new: bool,
}

impl TryFrom<UserViperSourceConfiguration> for ViperSourceDescriptor {
    type Error = ViperSourceMisconfigurationError;

    fn try_from(configuration: UserViperSourceConfiguration) -> Result<Self, Self::Error> {
        let name = configuration
            .name
            .right_ok_or(ViperSourceMisconfigurationError::InvalidSourceName)
            .and_then(|name| {
                ViperSourceName::try_from(name)
                    .map_err(|_| ViperSourceMisconfigurationError::InvalidSourceName)
            })?;

        let url = configuration
            .url
            .right_ok_or(ViperSourceMisconfigurationError::InvalidSourceUrl)
            .and_then(|url| {
                Url::parse(&url)
                    .map_err(|_| ViperSourceMisconfigurationError::InvalidSourceUrl)
            })?;

        let kind = configuration
            .kind
            .right_ok_or(ViperSourceMisconfigurationError::InvalidSourceKind)
            .and_then(|value| {
                match value.as_str() {
                    "Git" => Ok(ViperSourceKind::Git),
                    "HTTP" => Ok(ViperSourceKind::Http),
                    _ => Err(ViperSourceMisconfigurationError::InvalidSourceKind),
                }
            })?;

        Ok(
            ViperSourceDescriptor {
                id: configuration.id,
                name,
                url,
                kind,
            }
        )
    }
}
