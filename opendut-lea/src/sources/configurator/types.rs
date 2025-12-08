use url::Url;
use opendut_lea_components::UserInputValue;
use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperSourceName};

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum SourceMisconfigurationError {
    #[error("Invalid source name")]
    InvalidSourceName,
    #[error("Invalid source URL")]
    InvalidSourceUrl,
}

#[derive(Clone, Debug)]
pub struct UserSourceConfiguration {
    pub id: ViperSourceId,
    pub name: UserInputValue,
    pub url: UserInputValue,
    pub is_new: bool,
}

impl TryFrom<UserSourceConfiguration> for ViperSourceDescriptor {
    type Error = SourceMisconfigurationError;

    fn try_from(configuration: UserSourceConfiguration) -> Result<Self, Self::Error> {
        let name = configuration
            .name
            .right_ok_or(SourceMisconfigurationError::InvalidSourceName)
            .and_then(|name| {
                ViperSourceName::try_from(name)
                    .map_err(|_| SourceMisconfigurationError::InvalidSourceName)
            })?;

        let url = configuration
            .url
            .right_ok_or(SourceMisconfigurationError::InvalidSourceUrl)
            .and_then(|url| {
                Url::parse(&url)
                    .map_err(|_| SourceMisconfigurationError::InvalidSourceUrl)
            })?;

        Ok(
            ViperSourceDescriptor {
                id: configuration.id,
                name,
                url,
            }
        )
    }
}
