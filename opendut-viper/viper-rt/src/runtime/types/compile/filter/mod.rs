mod error;
pub use error::FilterError;

use crate::common::{Identifier, TestCaseIdentifier, TestIdentifier};

#[derive(Debug, Clone)]
pub struct IdentifierFilter {
    pub case_identifier: Option<String>,
    pub test_identifier: Option<String>,
}

impl IdentifierFilter {

    pub fn parse(identifier: &str) -> Self {
        let parts: Vec<&str> = identifier
            .split("::")
            .filter(|s| !s.is_empty())
            .collect();

        match parts.len() {
            0 | 1 => Self {
                case_identifier: None,
                test_identifier: None,
            },
            2 => {
                let case_identifier = parts.join("::").to_string();

                Self {
                    case_identifier: Some(case_identifier),
                    test_identifier: None,
                }
            }
            _ => {
                let case_identifier = parts[..2].join("::").to_string();
                let test_identifier = parts.join("::").to_string();
                Self {
                    case_identifier: Some(case_identifier),
                    test_identifier: Some(test_identifier),
                }
            }
        }
    }

    pub fn matches_case(&self, case_name: &TestCaseIdentifier) -> bool {
        self.case_identifier.as_ref()
            .map(|case| case == case_name.as_str())
            .unwrap_or(true)
    }

    pub fn matches_test(&self, test_name: &TestIdentifier) -> bool {
        self.test_identifier.as_ref()
            .map(|test| test == test_name.as_str())
            .unwrap_or(true)
    }
}
