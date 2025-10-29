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
}

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
}
