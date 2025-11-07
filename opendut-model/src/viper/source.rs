use std::fmt;
use std::ops::Not;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::create_id_type;


create_id_type!(ViperSourceId);


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ViperSourceName(pub(crate) String);

impl ViperSourceName {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 64;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalViperSourceName {
    #[error(
        "Test suite source name '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "Test suite source name '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("Test suite source name '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("Test suite source name '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<ViperSourceName> for String {
    fn from(value: ViperSourceName) -> Self {
        value.0
    }
}

impl TryFrom<String> for ViperSourceName {
    type Error = IllegalViperSourceName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalViperSourceName::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalViperSourceName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalViperSourceName::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_name(&c).not())
        {
            Err(IllegalViperSourceName::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ViperSourceName {
    type Error = IllegalViperSourceName;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ViperSourceName::try_from(value.to_owned())
    }
}

impl fmt::Display for ViperSourceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ViperSourceName {
    type Err = IllegalViperSourceName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViperSourceDescriptor {
    pub id: ViperSourceId,
    pub name: ViperSourceName,
    pub url: Url,
}
