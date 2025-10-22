#[derive(Debug)]
pub enum FilterError {
    InvalidTestCaseIdentifier {
        name: String
    },
    InvalidTestIdentifier {
        name: String
    },
}

impl FilterError {

    pub(crate) fn new_unknown_test_case_error(name: String) -> Self {
        Self::InvalidTestCaseIdentifier { name }
    }

    pub(crate) fn new_unknown_test_error(name: String) -> Self {
        Self::InvalidTestIdentifier { name }
    }
}
