use std::collections::HashMap;
use std::fmt;
use std::ops::Not;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use crate::cluster::ClusterId;
use crate::create_id_type;
use crate::viper::ViperSourceId;
use super::ViperTestSuiteIdentifier;


#[derive(Clone, Debug)]
pub struct ViperRunDescriptor {
    pub id: ViperRunId,
    pub name: ViperRunName,
    pub source: ViperSourceId,
    pub suite: ViperTestSuiteIdentifier,
    pub cluster: ClusterId,
    pub parameters: HashMap<ViperRunParameterKey, ViperRunParameterValue>,
}


create_id_type!(ViperRunId);


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ViperRunName(pub(crate) String);

impl ViperRunName {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 64;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalViperRunName {
    #[error(
        "VIPER run name '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "VIPER run name '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("VIPER run name '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("VIPER run name '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<ViperRunName> for String {
    fn from(value: ViperRunName) -> Self {
        value.0
    }
}

impl TryFrom<String> for ViperRunName {
    type Error = IllegalViperRunName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();

        if length < Self::MIN_LENGTH {
            Err(IllegalViperRunName::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        }
        else if length > Self::MAX_LENGTH {
            Err(IllegalViperRunName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        }
        else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalViperRunName::InvalidStartEndCharacter { value })
        }
        else if value
            .chars()
            .any(|character| crate::util::valid_characters_in_name(&character).not())
        {
            Err(IllegalViperRunName::InvalidCharacter { value })
        }
        else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ViperRunName {
    type Error = IllegalViperRunName;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ViperRunName::try_from(value.to_owned())
    }
}

impl fmt::Display for ViperRunName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ViperRunName {
    type Err = IllegalViperRunName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ViperRunParameterKey { pub inner: String }

#[derive(Clone, Debug)]
pub enum ViperRunParameterValue {
    Boolean(bool),
    Number(i64),
    Text(String),
}
