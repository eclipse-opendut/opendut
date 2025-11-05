use std::fmt;
use std::ops::Not;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::create_id_type;


create_id_type!(TestSuiteSourceId);


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TestSuiteSourceName(pub(crate) String);

impl TestSuiteSourceName {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 64;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalTestSuiteSourceName {
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

impl From<TestSuiteSourceName> for String {
    fn from(value: TestSuiteSourceName) -> Self {
        value.0
    }
}

impl TryFrom<String> for TestSuiteSourceName {
    type Error = IllegalTestSuiteSourceName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalTestSuiteSourceName::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalTestSuiteSourceName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalTestSuiteSourceName::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_name(&c).not())
        {
            Err(IllegalTestSuiteSourceName::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for TestSuiteSourceName {
    type Error = IllegalTestSuiteSourceName;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        TestSuiteSourceName::try_from(value.to_owned())
    }
}

impl fmt::Display for TestSuiteSourceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TestSuiteSourceName {
    type Err = IllegalTestSuiteSourceName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestSuiteSourceDescriptor {
    pub id: TestSuiteSourceId,
    pub name: TestSuiteSourceName,
    pub url: Url,
}
