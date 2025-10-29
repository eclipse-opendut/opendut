mod error;

pub use error::FilterError;

use crate::common::{TestCaseIdentifier, TestIdentifier, TestSuiteIdentifier};

#[derive(Debug, Clone, Default)]
pub struct IdentifierFilter {
    pub suite_identifier: Option<TestSuiteIdentifier>,
    pub case_identifier: Option<TestCaseIdentifier>,
    pub test_identifier: Option<TestIdentifier>,
}

impl IdentifierFilter {

    pub fn parse(identifier: &str) -> Result<Self, FilterError> {
        let parts: Vec<&str> = identifier
            .split("::")
            .filter(|s| !s.is_empty())
            .collect();

        match parts.len() {
            0 => Ok(Self::default()),
            1 => {
                let suite_identifier = Self::get_suite_identifier(&parts)?;

                Ok(Self {
                    suite_identifier: Some(suite_identifier),
                    ..Default::default()
                })
            }
            2 => {
                let suite_identifier = Self::get_suite_identifier(&parts)?;
                let case_identifier = Self::get_case_identifier(&parts)?;

                Ok(Self {
                    suite_identifier: Some(suite_identifier),
                    case_identifier: Some(case_identifier),
                    test_identifier: None,
                })
            }
            _ => {
                let suite_identifier = Self::get_suite_identifier(&parts)?;
                let case_identifier = Self::get_case_identifier(&parts)?;
                let test_identifier = Self::get_test_identifier(&parts)?;

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

    fn get_suite_identifier(parts: &Vec<&str>) -> Result<TestSuiteIdentifier, FilterError> {
        TestSuiteIdentifier::try_from(parts[0])
            .map_err(|err| FilterError::new_invalid_test_suite_filter_error(err))
    }

    fn get_case_identifier(parts: &Vec<&str>) -> Result<TestCaseIdentifier, FilterError> {
        TestCaseIdentifier::try_from(parts[..2].join("::").to_string())
            .map_err(|err| FilterError::new_invalid_test_case_filter_error(err))
    }

    fn get_test_identifier(parts: &Vec<&str>) -> Result<TestIdentifier, FilterError> {
        TestIdentifier::try_from(parts.join("::").to_string())
            .map_err(|err| FilterError::new_invalid_test_filter_error(err))
    }
}
