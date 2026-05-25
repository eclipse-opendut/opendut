use std::fmt;
use std::ops::Not;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::create_id_type;


create_id_type!(SecretId);


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretName(pub(crate) String);

impl SecretName {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 64;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalSecretName {
    #[error(
        "Secret name '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "Secret name '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("Secret name '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("Secret name '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<SecretName> for String {
    fn from(value: SecretName) -> Self {
        value.0
    }
}

impl TryFrom<String> for SecretName {
    type Error = IllegalSecretName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalSecretName::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalSecretName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalSecretName::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_name(&c).not())
        {
            Err(IllegalSecretName::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for SecretName {
    type Error = IllegalSecretName;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        SecretName::try_from(value.to_owned())
    }
}

impl fmt::Display for SecretName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SecretName {
    type Err = IllegalSecretName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretValue {
    Token(String),
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretDescriptor {
    pub id: SecretId,
    pub name: SecretName,
    pub value: SecretValue,
}
