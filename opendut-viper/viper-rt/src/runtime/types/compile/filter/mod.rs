mod error;

pub use error::FilterError;

use crate::common::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};
use crate::runtime::types::naming::error::InvalidIdentifierError;

#[derive(Debug, Clone, Default)]
pub struct IdentifierFilter {
    pub suite_identifier: Option<TestSuiteIdentifier>,
    pub case_identifier: Option<TestCaseIdentifier>,
    pub test_identifier: Option<TestIdentifier>,
}

impl IdentifierFilter {

    pub fn parse(identifier: &str) -> Result<Self, InvalidIdentifierError> {
        let parts: Vec<&str> = identifier
            .split("::")
            .filter(|s| !s.is_empty())
            .collect();

        match parts.len() {
            0 => Ok(Self::default()),
            1 => {
                Ok(Self {
                    suite_identifier: None,
                    ..Default::default()
                })
            }
            2 => {
                let suite_identifier = TestSuiteIdentifier::try_from(parts[0])?;
                let case_identifier = TestCaseIdentifier::try_from(parts.join("::").to_string())?;

                Ok(Self {
                    suite_identifier: Some(suite_identifier),
                    case_identifier: Some(case_identifier),
                    test_identifier: None,
                })
            }
            _ => {
                let suite_identifier = TestSuiteIdentifier::try_from(parts[0])?;
                let case_identifier = TestCaseIdentifier::try_from(parts[..2].join("::").to_string())?;
                let test_identifier = TestIdentifier::try_from(parts.join("::").to_string())?;

                Ok(Self {
                    suite_identifier: Some(suite_identifier),
                    case_identifier: Some(case_identifier),
                    test_identifier: Some(test_identifier),
                })
            }
        }
    }

    pub fn matches_suite(&self, suite_name: &TestSuiteIdentifier) -> bool{
        self.suite_identifier.as_ref()
            .map(|suite| suite == suite_name)
            .unwrap_or(true)
    }

    pub fn matches_case(&self, case_name: &TestCaseIdentifier) -> bool {
        self.case_identifier.as_ref()
            .map(|case| case == case_name)
            .unwrap_or(true)
    }

    pub fn matches_test(&self, test_name: &TestIdentifier) -> bool {
        self.test_identifier.as_ref()
            .map(|test| test == test_name)
            .unwrap_or(true)
    }
}
