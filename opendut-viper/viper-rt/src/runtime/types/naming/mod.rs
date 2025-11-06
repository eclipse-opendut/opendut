//! Naming things is hard. Therefore, this module provides structures and utilities to represent
//! hierarchical identifiers, including:
//!
//! - [`TestSuiteIdentifier`] for test-suite names.
//! - [`TestCaseIdentifier`] for test-case names.
//! - [`TestIdentifier`] for test names.
//!
//! This module includes also functionality for constructing and querying these identifiers, as well
//! as implementing various string-related traits for convenience.
//!
pub mod error;
mod validate;

use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::ops::{Not, Range};
use crate::runtime::types::naming::error::InvalidIdentifierError;

const SEPARATOR: &str = "::";

pub trait Identifier : Debug + Display {

    /// Extracts a string slice containing the entire identifier.
    fn as_str(&self) -> &str;

    fn name(&self) -> &str;
}

/// This identifier is used to identify a test-suite.
///
/// # Example
/// ```
/// use std::path::{absolute, PathBuf};
/// use opendut_viper_rt::source::loaders::SimpleFileSourceLoader;
/// use opendut_viper_rt::source::Source;
/// use opendut_viper_rt::events::emitter;
/// use opendut_viper_rt::ViperRuntime;
/// use opendut_viper_rt::compile::IdentifierFilter;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let runtime = ViperRuntime::builder()
///         .with_source_loader(SimpleFileSourceLoader)
///         .build()?;
///
///     let path = absolute(PathBuf::from("tests/minimal.py"))?;
///
///     let source = Source::try_from_path("minimal".try_into()?, &path)?;
///     let suite = runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.into_suite();
///
///     assert_eq!(suite.name(), "minimal");
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TestSuiteIdentifier {
    value: String
}

impl TestSuiteIdentifier {

    pub(crate) fn new(name: String) -> Self {
        assert!(name.is_empty().not(), "The name for a test-suite must not be empty");
        assert!(validate::invalid_test_suite_identifier_characters(&name).is_ok(), "The name for a test-suite must not contain invalid characters: {name}");
        Self::new_unchecked(name)
    }

    fn new_unchecked(name: impl Into<String>) -> Self {
        Self { value: name.into() }
    }

    pub(crate) fn new_embedded() -> Self {
        Self { value: String::from("_embedded_") }
    }
}

/// This identifier is used to identify a test-case.
///
/// # Example
/// ```
/// use opendut_viper_rt::source::Source;
/// use opendut_viper_rt::ViperRuntime;
/// use opendut_viper_rt::events::emitter;
/// use opendut_viper_rt::compile::IdentifierFilter;
/// use indoc::indoc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let runtime = ViperRuntime::default();
///
///     let source = Source::embedded(indoc!(r#"
///         ## VIPER_VERSION = 1.0
///         from viper import unittest
///
///         class MyTestCase(unittest.TestCase):
///             pass
///     "#));
///
///     let suite = runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.into_suite();
///     let case = &suite.test_cases()[0];
///
///     assert_eq!(case.identifier(), "_embedded_::MyTestCase");
///     assert_eq!(case.name(), "MyTestCase");
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TestCaseIdentifier {
    value: String,
    suite: Range<usize>,
    case: Range<usize>
}

impl TestCaseIdentifier {

    pub fn try_from_suite(suite: &TestSuiteIdentifier, name: &str) -> Result<Self, InvalidIdentifierError> {
        if name.is_empty() {
            return Err(InvalidIdentifierError::new_empty_identifier_error())
        }
        validate::invalid_test_case_identifier_characters(name)?;
        Ok(Self::new_unchecked(suite.as_str(), name))
    }

    pub(crate) fn new(suite: &TestSuiteIdentifier, name: &str) -> Self {

        assert!(name.is_empty().not(), "The name for a test-case must not be empty");
        assert!(validate::invalid_test_case_identifier_characters(name).is_ok(), "The name for a test-case must not contain invalid characters: {name}");

        Self::new_unchecked(suite.name(), name)
    }

    fn new_unchecked(suite_name: &str, case_name: &str) -> Self {

        let mut suite_range = Range::default();
        let mut case_range = Range::default();
        let mut value = String::new();

        value.push_str(suite_name);
        suite_range.end = value.len();
        value.push_str(SEPARATOR);
        case_range.start = value.len();
        value.push_str(case_name);
        case_range.end = value.len();

        Self { value, suite: suite_range, case: case_range }
    }

    pub fn suite(&self) -> TestSuiteIdentifier {
        TestSuiteIdentifier::new(self.suite_str().to_string())
    }

    pub fn suite_str(&self) -> &str {
        &self.value[self.suite.start..self.suite.end]
    }
}

/// This identifier is used to identify a test.
///
/// # Example
/// ```
/// use opendut_viper_rt::source::Source;
/// use opendut_viper_rt::ViperRuntime;
/// use opendut_viper_rt::events::emitter;
/// use opendut_viper_rt::compile::IdentifierFilter;
/// use indoc::indoc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let runtime = ViperRuntime::default();
///
///     let source = Source::embedded(indoc!(r#"
///         ## VIPER_VERSION = 1.0
///         from viper import unittest
///
///         class MyTestCase(unittest.TestCase):
///             def test_awesomeness(self):
///                 pass
///     "#));
///
///     let suite = runtime.compile(&source, &mut emitter::drain(), &IdentifierFilter::default()).await?.into_suite();
///     let test = &suite.test_cases()[0].tests()[0];
///
///     assert_eq!(test.identifier(), "_embedded_::MyTestCase::test_awesomeness");
///     assert_eq!(test.name(), "test_awesomeness");
///
///     Ok(())
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TestIdentifier {
    value: String,
    suite_name_range: Range<usize>,
    case_name_range: Range<usize>,
    test_name_range: Range<usize>
}

impl TestIdentifier {

    pub fn try_from_case(case: &TestCaseIdentifier, name: &str) -> Result<Self, InvalidIdentifierError> {
        if name.is_empty() {
            return Err(InvalidIdentifierError::new_empty_identifier_error())
        }
        validate::invalid_test_identifier_characters(name)?;
        Ok(Self::new_unchecked(case.suite_str(), case.name(), name))
    }

    pub(crate) fn new(case: &TestCaseIdentifier, name: &str) -> Self {

        assert!(name.is_empty().not(), "The name for a test must not be empty");
        assert!(validate::invalid_test_identifier_characters(name).is_ok(), "The name for a test must not contain invalid characters: {name}");

        Self::new_unchecked(case.suite_str(), case.name(), name)
    }

    fn new_unchecked(suite_name: &str, case_name: &str, test_name: &str) -> Self {
        let mut suite_name_range = Range::default();
        let mut case_name_range = Range::default();
        let mut test_name_range = Range::default();
        let mut value = String::new();

        value.push_str(suite_name);
        suite_name_range.end = value.len();
        value.push_str(SEPARATOR);
        case_name_range.start = value.len();
        value.push_str(case_name);
        case_name_range.end = value.len();
        value.push_str(SEPARATOR);
        test_name_range.start = value.len();
        value.push_str(test_name);
        test_name_range.end = value.len();

        Self { value, suite_name_range, case_name_range, test_name_range }
    }

    pub fn suite_str(&self) -> &str {
        &self.value[self.suite_name_range.start..self.suite_name_range.end]
    }

    pub fn case_str(&self) -> &str {
        &self.value[self.case_name_range.start..self.case_name_range.end]
    }
}

impl Identifier for TestSuiteIdentifier {

    fn as_str(&self) -> &str {
        self.value.as_str()
    }

    fn name(&self) -> &str {
        self.value.as_str()
    }
}

impl Identifier for TestCaseIdentifier {

    fn as_str(&self) -> &str {
        self.value.as_str()
    }

    fn name(&self) -> &str {
        &self.value[self.case.start..self.case.end]
    }
}

impl Identifier for TestIdentifier {

    fn as_str(&self) -> &str {
        self.value.as_str()
    }

    fn name(&self) -> &str {
        &self.value[self.test_name_range.start..self.test_name_range.end]
    }
}

impl Display for TestSuiteIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for TestCaseIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Display for TestIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<String> for TestSuiteIdentifier {

    type Error = InvalidIdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {

        if value.is_empty() {
            return Err(InvalidIdentifierError::new_empty_identifier_error());
        }

        validate::invalid_test_suite_identifier_characters(&value)?;

        Ok(Self { value })
    }
}

impl TryFrom<&str> for TestSuiteIdentifier {
    type Error = InvalidIdentifierError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(String::from(value))
    }
}

impl TryFrom<Cow<'_, str>> for TestSuiteIdentifier {
    type Error = InvalidIdentifierError;
    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        Self::try_from(value.as_ref())
    }
}

impl TryFrom<&str> for TestCaseIdentifier {

    type Error = InvalidIdentifierError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {

        if value.is_empty() {
            return Err(InvalidIdentifierError::new_empty_identifier_error());
        }

        let mut parts = value.split(SEPARATOR);

        let suite_part = {
            let part = parts.next();
            if part.is_none_or(str::is_empty) {
                return Err(InvalidIdentifierError::new_missing_test_suite_identifier_error(value))
            }
            validate::invalid_test_suite_identifier_characters(part.expect("The suite part has to be checked for not be none until here"))?
        };

        let case_part = {
            let part = parts.next();
            if part.is_none_or(str::is_empty) {
                return Err(InvalidIdentifierError::new_missing_test_case_identifier_error(value))
            }
            validate::invalid_test_case_identifier_characters(part.expect("The case part has to be checked for not be none until here"))?
        };

        if let Some(remaining) = parts.next() {
            return Err(InvalidIdentifierError::new_unexpected_remaining_characters_error(value, remaining))
        }

        Ok(TestCaseIdentifier::new_unchecked(suite_part, case_part))
    }
}

impl TryFrom<String> for TestCaseIdentifier {

    type Error = InvalidIdentifierError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(value.as_str())
    }
}

impl TryFrom<Cow<'_, str>> for TestCaseIdentifier {
    type Error = InvalidIdentifierError;
    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(value.as_ref())
    }
}

impl TryFrom<&str> for TestIdentifier {

    type Error = InvalidIdentifierError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {

        if value.is_empty() {
            return Err(InvalidIdentifierError::new_empty_identifier_error());
        }

        let mut parts = value.split(SEPARATOR);

        let suite_part = {
            let part = parts.next()
                .ok_or_else(|| InvalidIdentifierError::new_missing_test_suite_identifier_error(value))?;
            validate::invalid_test_suite_identifier_characters(part)?
        };

        let case_part = {
            let part = parts.next()
                .ok_or_else(|| InvalidIdentifierError::new_missing_test_case_identifier_error(value))?;
            validate::invalid_test_case_identifier_characters(part)?
        };

        let test_part = {
            let part = parts.next()
                .ok_or_else(|| InvalidIdentifierError::new_missing_test_identifier_error(value))?;
            validate::invalid_test_identifier_characters(part)?
        };

        if let Some(remaining) = parts.next() {
            return Err(InvalidIdentifierError::new_unexpected_remaining_characters_error(value, remaining))
        }

        Ok(TestIdentifier::new_unchecked(suite_part, case_part, test_part))
    }
}

impl TryFrom<String> for TestIdentifier {
    type Error = InvalidIdentifierError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(value.as_str())
    }
}

impl TryFrom<Cow<'_, str>> for TestIdentifier {
    type Error = InvalidIdentifierError;
    fn try_from(value: Cow<'_, str>) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(value.as_ref())
    }
}

impl From<TestSuiteIdentifier> for String {
    fn from(value: TestSuiteIdentifier) -> Self {
        value.value
    }
}

impl From<TestCaseIdentifier> for String {
    fn from(value: TestCaseIdentifier) -> Self {
        value.value
    }
}

impl From<TestIdentifier> for String {
    fn from(value: TestIdentifier) -> Self {
        value.value
    }
}

impl PartialEq for TestSuiteIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq for TestCaseIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq for TestIdentifier {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq<String> for TestSuiteIdentifier {
    fn eq(&self, other: &String) -> bool {
        &self.value == other
    }
}

impl PartialEq<String> for TestCaseIdentifier {
    fn eq(&self, other: &String) -> bool {
        &self.value == other
    }
}

impl PartialEq<String> for TestIdentifier {
    fn eq(&self, other: &String) -> bool {
        &self.value == other
    }
}

impl PartialEq<str> for TestSuiteIdentifier {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

impl PartialEq<str> for TestCaseIdentifier {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

impl PartialEq<str> for TestIdentifier {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use super::*;
    use googletest::prelude::*;

    #[test]
    fn test_TestSuiteIdentifier_new() -> Result<()> {

        let suite = TestSuiteIdentifier::new(String::from("awesome.py"));

        assert_that!(suite, eq("awesome.py"));
        assert_that!(suite.as_str(), eq("awesome.py"));
        assert_that!(suite.name(), eq("awesome.py"));

        Ok(())
    }

    #[test]
    fn test_TestCaseIdentifier_new() -> Result<()> {

        let suite = TestSuiteIdentifier::new(String::from("awesome.py"));
        let case = TestCaseIdentifier::new(&suite, "MyAwesomeTestCase");

        assert_that!(case, eq("awesome.py::MyAwesomeTestCase"));
        assert_that!(case.as_str(), eq("awesome.py::MyAwesomeTestCase"));
        assert_that!(case.suite_str(), eq("awesome.py"));
        assert_that!(case.name(), eq("MyAwesomeTestCase"));

        Ok(())
    }

    #[test]
    fn test_TestIdentifiers_new() -> Result<()> {

        let suite = TestSuiteIdentifier::new(String::from("awesome.py"));
        let case = TestCaseIdentifier::new(&suite, "MyAwesomeTestCase");
        let test = TestIdentifier::new(&case, "test_awesomeness");

        assert_that!(test, eq("awesome.py::MyAwesomeTestCase::test_awesomeness"));
        assert_that!(test.as_str(), eq("awesome.py::MyAwesomeTestCase::test_awesomeness"));
        assert_that!(test.suite_str(), eq("awesome.py"));
        assert_that!(test.case_str(), eq("MyAwesomeTestCase"));
        assert_that!(test.name(), eq("test_awesomeness"));

        Ok(())
    }
}
