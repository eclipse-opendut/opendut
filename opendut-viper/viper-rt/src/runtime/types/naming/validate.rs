use std::ops::Not;
use crate::runtime::types::naming::error::InvalidIdentifierError;
use crate::runtime::types::naming::TestSuiteIdentifier;

const ALLOWED_TEST_SUITE_IDENTIFIER_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-.";
const ALLOWED_TEST_CASE_IDENTIFIER_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
const ALLOWED_TEST_IDENTIFIER_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";

pub fn test_suite_identifier(value: &str) -> Result<&str, InvalidIdentifierError> {
    let length = value.len();

    if length > TestSuiteIdentifier::MAX_LENGTH {
        Err(InvalidIdentifierError::new_test_suite_identifier_too_long_error(value, TestSuiteIdentifier::MAX_LENGTH, length))
    }
    else if let Some(char) = find_invalid_character(value, ALLOWED_TEST_SUITE_IDENTIFIER_CHARS) {
        Err(InvalidIdentifierError::new_illegal_test_suite_identifier_character_error(value, char))
    }
    else {
        Ok(value)
    }
}

pub fn invalid_test_case_identifier_characters(value: &str) -> Result<&str, InvalidIdentifierError> {
    if let Some(char) = find_invalid_character(value, ALLOWED_TEST_CASE_IDENTIFIER_CHARS) {
        Err(InvalidIdentifierError::new_illegal_test_case_identifier_character_error(value, char))
    }
    else {
        Ok(value)
    }
}

pub fn invalid_test_identifier_characters(value: &str) -> Result<&str, InvalidIdentifierError> {
    if let Some(char) = find_invalid_character(value, ALLOWED_TEST_IDENTIFIER_CHARS) {
        Err(InvalidIdentifierError::new_illegal_test_identifier_character_error(value, char))
    }
    else {
        Ok(value)
    }
}

fn find_invalid_character(value: &str, allowed_chars: &str) -> Option<char> {
    value.chars().find(|char| allowed_chars.contains(*char).not())
}
