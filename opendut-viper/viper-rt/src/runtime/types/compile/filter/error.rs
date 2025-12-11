use crate::runtime::types::naming::error::InvalidIdentifierError;

#[derive(Debug)]
pub enum FilterError {
    TestSuiteNotFound {
        name: String
    },
    TestCaseNotFound {
        name: String
    },
    TestNotFound {
        name: String
    },
    InvalidTestSuiteFilter {
        case: InvalidIdentifierError
    },
    InvalidTestCaseFilter {
        case: InvalidIdentifierError
    },
    InvalidTestFilter {
        case: InvalidIdentifierError
    }
}

#[allow(dead_code)]
impl FilterError {

    pub(crate) fn new_test_suite_not_found_error(name: String) -> Self {
        Self::TestSuiteNotFound { name }
    }

    pub(crate) fn new_test_case_not_found_error(name: String) -> Self {
        Self::TestCaseNotFound { name }
    }

    pub(crate) fn new_test_not_found_error(name: String) -> Self {
        Self::TestNotFound { name }
    }

    pub(crate) fn new_invalid_test_suite_filter_error(case: InvalidIdentifierError) -> Self {
        Self::InvalidTestSuiteFilter { case }
    }

    pub(crate) fn new_invalid_test_case_filter_error(case: InvalidIdentifierError) -> Self {
        Self::InvalidTestCaseFilter { case }
    }

    pub(crate) fn new_invalid_test_filter_error(case: InvalidIdentifierError) -> Self {
        Self::InvalidTestFilter { case }
    }
}
