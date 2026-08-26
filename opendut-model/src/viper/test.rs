use std::fmt;
use std::ops::Not;
use std::str::FromStr;
use opendut_viper_rt::compile::ParameterName;
use opendut_viper_rt::run::{BindingValue};
use serde::{Deserialize, Serialize};
use crate::create_id_type;
use crate::peer::PeerId;
use crate::viper::ViperSourceId;


/// Information how a test run should be configured when executed via VIPER.
#[derive(Clone, Debug, PartialEq)]
pub struct ViperTestRunDescriptor {
    pub id: ViperTestId,
    pub name: ViperTestName,
    pub source: ViperSourceId,
    pub peer: PeerId,
    /// Concrete values bindings to run the test with.
    pub parameters: ViperTestParameters, // not including ParameterDescriptors here, because they could become out-of-date when we persist them
}


create_id_type!(ViperTestId);


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ViperTestName(pub(crate) String);

impl ViperTestName {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 64;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalViperTestName {
    #[error(
        "VIPER test name '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "VIPER test name '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("VIPER test name '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("VIPER test name '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<ViperTestName> for String {
    fn from(value: ViperTestName) -> Self {
        value.0
    }
}

impl TryFrom<String> for ViperTestName {
    type Error = IllegalViperTestName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();

        if length < Self::MIN_LENGTH {
            Err(IllegalViperTestName::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        }
        else if length > Self::MAX_LENGTH {
            Err(IllegalViperTestName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        }
        else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalViperTestName::InvalidStartEndCharacter { value })
        }
        else if value
            .chars()
            .any(|character| crate::util::valid_characters_in_name(&character).not())
        {
            Err(IllegalViperTestName::InvalidCharacter { value })
        }
        else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for ViperTestName {
    type Error = IllegalViperTestName;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        ViperTestName::try_from(value.to_owned())
    }
}

impl fmt::Display for ViperTestName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ViperTestName {
    type Err = IllegalViperTestName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

/// Maps to `ParameterBindings` in VIPER.
pub type ViperTestParameters = Vec<(ParameterName, Option<BindingValue>)>;
