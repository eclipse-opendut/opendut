#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub struct InvalidIdentifierError {
    pub value: String,
    pub kind: InvalidIdentifierErrorKind
}

#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum InvalidIdentifierErrorKind {
    Empty,
    IllegalTestSuiteIdentifierCharacter { character: char },
    IllegalTestCaseIdentifierCharacter { character: char },
    IllegalTestIdentifierCharacter { character: char },
    MissingTestSuiteIdentifier,
    MissingTestCaseIdentifier,
    MissingTestIdentifier,
    UnexpectedRemainingCharacters { remaining: String },
}

impl InvalidIdentifierError {

    pub fn new_empty_identifier_error() -> Self {
        Self { value: String::new(), kind: InvalidIdentifierErrorKind::Empty }
    }

    pub fn new_illegal_test_suite_identifier_character_error(value: impl Into<String>, character: char) -> Self {
        Self { value: value.into(), kind: InvalidIdentifierErrorKind::IllegalTestSuiteIdentifierCharacter { character } }
    }

    pub fn new_illegal_test_case_identifier_character_error(value: impl Into<String>, character: char) -> Self {
        Self { value: value.into(), kind: InvalidIdentifierErrorKind::IllegalTestCaseIdentifierCharacter { character } }
    }

    pub fn new_illegal_test_identifier_character_error(value: impl Into<String>, character: char) -> Self {
        Self { value: value.into(), kind: InvalidIdentifierErrorKind::IllegalTestIdentifierCharacter { character } }
    }

    pub fn new_unexpected_remaining_characters_error(value: impl Into<String>, remaining: impl Into<String>) -> Self {
        Self { value: value.into(), kind: InvalidIdentifierErrorKind::UnexpectedRemainingCharacters { remaining: remaining.into() } }
    }

    pub fn new_missing_test_suite_identifier_error(value: impl Into<String>) -> Self {
        Self { value: value.into(), kind: InvalidIdentifierErrorKind::MissingTestSuiteIdentifier }
    }

    pub fn new_missing_test_case_identifier_error(value: impl Into<String>) -> Self {
        Self { value: value.into(), kind: InvalidIdentifierErrorKind::MissingTestCaseIdentifier }
    }

    pub fn new_missing_test_identifier_error(value: impl Into<String>) -> Self {
        Self { value: value.into(), kind: InvalidIdentifierErrorKind::MissingTestIdentifier }
    }
}
