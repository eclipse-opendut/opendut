mod error;

pub use error::SuiteFilterError;

use viper_rt::common::{Identifier, TestSuiteIdentifier};

#[derive(Debug, Clone)]
pub struct SuiteIdentifierFilter {
    pub suite_identifier: Option<String>,
}

impl SuiteIdentifierFilter {

    pub fn parse(identifier: &str) -> Self {
        let parts: Vec<&str> = identifier
            .split("::")
            .filter(|s| !s.is_empty())
            .collect();

        match parts.len() {
            0 => Self {
                suite_identifier: None,
            },
            _ => {
                let suite_identifier = parts[0].to_string();
                Self {
                    suite_identifier: Some(suite_identifier),
                }
            }
        }
    }

    pub fn matches_suite(&self, suite_name: &TestSuiteIdentifier) -> bool {
        self.suite_identifier.as_ref()
            .map(|suite| suite == suite_name.as_str())
            .unwrap_or(true)
    }
}
