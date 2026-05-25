use opendut_lea_components::UserInputValue;
use opendut_model::secret::{SecretDescriptor, SecretId, SecretName, SecretValue};

#[derive(thiserror::Error, Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum SecretMisconfigurationError {
    #[error("Invalid secret name")]
    InvalidSecretName,
    #[error("Invalid secret value")]
    InvalidSecretValue,
}

#[derive(Clone, Debug)]
pub struct UserSecretConfiguration {
    pub id: SecretId,
    pub name: UserInputValue,
    pub value: UserInputValue,
    pub is_new: bool,
}

impl UserSecretConfiguration {
    pub fn is_valid(&self) -> bool {
        self.name.is_right() && self.value.is_right()
    }
}

impl TryFrom<UserSecretConfiguration> for SecretDescriptor {
    type Error = SecretMisconfigurationError;

    fn try_from(configuration: UserSecretConfiguration) -> Result<Self, Self::Error> {
        let name = configuration
            .name
            .right_ok_or(SecretMisconfigurationError::InvalidSecretName)
            .and_then(|name| {
                SecretName::try_from(name)
                    .map_err(|_| SecretMisconfigurationError::InvalidSecretName)
            })?;

        let value = configuration
            .value
            .right_ok_or(SecretMisconfigurationError::InvalidSecretValue)
            .map(SecretValue::Token)?;

        Ok(
            SecretDescriptor {
                id: configuration.id,
                name,
                value,
            }
        )
    }
}
