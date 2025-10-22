use std::fmt;

#[derive(Debug)]
pub enum SuiteFilterError {
    InvalidSuiteIdentifier {
        name: String
    },
}

impl SuiteFilterError {
    pub(crate) fn new_unknown_test_suite_error(name: String) -> Self {
        Self::InvalidSuiteIdentifier { name }
    }
}

impl fmt::Display for SuiteFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuiteFilterError::InvalidSuiteIdentifier { name } => {
                write!(f, "TestSuite '{name}' not found.")
            }
        }
    }
}

impl std::error::Error for SuiteFilterError {}
