#![cfg(feature = "error")]

use crate::runtime::types::compile::error::CompilationError;
use crate::runtime::types::compile::error::CompilationErrorKind;
use crate::runtime::types::compile::inspect::InspectionError;
use crate::runtime::types::compile::metadata::MetadataError;
use crate::runtime::types::compile::parameters::{InvalidParameterNameError, ParameterError};
use crate::runtime::types::naming::error::{InvalidIdentifierError, InvalidIdentifierErrorKind};
use crate::runtime::types::py::error::{PythonReflectionError, PythonRuntimeError};
use crate::runtime::types::run::error::RunError;
use crate::runtime::types::run::error::RunErrorKind;
use crate::runtime::types::source::error::{InvalidSourceError, InvalidSourceLocationError};
use crate::runtime::RuntimeInstantiationError;
use crate::source::SourceLocation;
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::runtime::types::compile::filter::FilterError;
use crate::runtime::types::run::parameters::{BindParameterError, IncompleteParameterBindingsError};

impl Error for CompilationError {}
impl Error for BindParameterError {}
impl Error for IncompleteParameterBindingsError {}
#[cfg(feature = "events")]
impl Error for crate::events::EventEmissionError {}
impl Error for InvalidParameterNameError {}
impl Error for InspectionError {}
impl Error for InvalidIdentifierError {}
impl Error for InvalidSourceError {}
impl Error for InvalidSourceLocationError {}
impl Error for MetadataError {}
impl Error for FilterError {}
impl Error for ParameterError {}
impl Error for PythonReflectionError {}
impl Error for PythonRuntimeError {}
impl Error for RunError {}
impl Error for RuntimeInstantiationError {}

impl Display for CompilationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let source_name = &self.identifier;
        match &self.kind {
            CompilationErrorKind::FailedEventEmission { message } => {
                write!(f, "Compilation failed, due to an event emission error: {message}")
            }
            CompilationErrorKind::FailedLoading { source_location, cause } => {
                match source_location {
                    SourceLocation::Embedded(_) => {
                        panic!("This should never happen because embedded sources must not be loaded.")
                    }
                    SourceLocation::Url(url) => {
                        write!(f, "Compilation failed because source '{source_name}' could not be loaded from '{url}': '{cause}'")
                    }
                }
            }
            CompilationErrorKind::InvalidSource { cause } => {
                write!(f, "Compilation failed, due to an invalid source '{source_name}': {cause}")
            }
            CompilationErrorKind::NoSuitableSourceLoader { source_location } => {
                match source_location {
                    SourceLocation::Embedded(_) => {
                        write!(f, "Compilation failed because there is no `SourceLoader` registered for embedded sources.")
                    }
                    SourceLocation::Url(url) => {
                        write!(f, "Compilation failed because there is no `SourceLoader` registered to load '{source_name}' from '{url}'.")
                    }
                }
            }
            CompilationErrorKind::FailedInspection { cause } => {
                write!(f, "Compilation failed during inspection of source '{source_name}': {cause}")
            }
            CompilationErrorKind::PythonCompilationError { details } => {
                write!(f, "Compilation failed, due to a Python compilation error in source '{source_name}': {details}")
            }
            CompilationErrorKind::PythonReflectionError { cause } => {
                write!(f, "Compilation failed, due to a Python reflection error in source '{source_name}': {cause}")
            }
            CompilationErrorKind::PythonRuntimeError { cause } => {
                write!(f, "Compilation failed, due to a Python runtime error in source '{source_name}': {cause}")
            }
        }
    }
}

impl Display for BindParameterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BindParameterError::ParameterNotFound(name) =>
                write!(f, "Parameter '{name}' not found to bind value to!"),
            BindParameterError::TypeMismatch{ parameter_name, expected_type, actual_type} =>
                write!(f, "Expected binding value of type '{expected_type}' for parameter '{parameter_name}', but got value of type '{actual_type}'!"),
            BindParameterError::NumberValueOutOfRange { parameter_name, value, min, max } =>
                write!(f, "Value {value} for number parameter '{parameter_name}' is out of range [{min}, {max}]!"),
            BindParameterError::TextValueOutOfRange { parameter_name, value, max } =>
                write!(f, "Value for text parameter '{parameter_name}' exceeds the maximum length of {max} characters ({}): {value}", value.len()),
        }
    }
}

impl Display for IncompleteParameterBindingsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Incomplete parameter bindings! No value is assigned to the following parameters: {}", self.missing_parameters.iter().map(String::from).collect::<Vec<_>>().join(", "))
    }
}

#[cfg(feature = "events")]
impl Display for crate::events::EventEmissionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Event emission failed.")
    }
}

impl Display for InvalidParameterNameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Illegal value for parameter name: '{}'", self.value)
    }
}

impl Display for InspectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InspectionError::MetadataError(cause) => write!(f, "{cause}"),
            InspectionError::ParameterError(cause) => write!(f, "{cause}"),
            InspectionError::FilterError(cause) => write!(f, "{cause}"),
        }
    }
}

impl Display for InvalidIdentifierError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let InvalidIdentifierError { value, kind } = self;
        match kind {
            InvalidIdentifierErrorKind::Empty => write!(f, "Identifier is empty."),
            InvalidIdentifierErrorKind::IllegalTestSuiteIdentifierCharacter { character } => write!(f, "Value '{value}' contains illegal character '{character}' for a test suite name."),
            InvalidIdentifierErrorKind::IllegalTestCaseIdentifierCharacter { character } => write!(f, "Value '{value}' contains illegal character '{character}' for a test case name."),
            InvalidIdentifierErrorKind::IllegalTestIdentifierCharacter { character } => write!(f, "Value '{value}' contains illegal character '{character}' for a test name."),
            InvalidIdentifierErrorKind::MissingTestSuiteIdentifier => write!(f, "Identifier '{value}' is missing a test suite name."),
            InvalidIdentifierErrorKind::MissingTestCaseIdentifier => write!(f, "Identifier '{value}' is missing a test case name."),
            InvalidIdentifierErrorKind::MissingTestIdentifier => write!(f, "Identifier '{value}' is missing a test name."),
            InvalidIdentifierErrorKind::UnexpectedRemainingCharacters { remaining } => write!(f, "Value '{value}' contains unexpected remaining characters: '{remaining}'"),
        }
    }
}

impl Display for InvalidSourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidSourceError::EmptySource => write!(f, "Source is empty."),
            InvalidSourceError::MissingViperVersion => write!(f, "First line does not contain `VIPER_VERSION`."),
            InvalidSourceError::IllegalViperVersionString(value) => write!(f, "Illegal `VIPER_VERSION` string: {value}"),
            InvalidSourceError::UnknownViperVersion(value) => write!(f, "Unknown `VIPER_VERSION`: {value}"),
        }
    }
}

impl Display for InvalidSourceLocationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            InvalidSourceLocationError::InvalidUrl { url, message, .. } => write!(f, "Invalid url '{url}': {message}"),
            InvalidSourceLocationError::NonAbsolutePath { path, .. } => write!(f, "Invalid path '{}'. Path must be absolute!", path.display()),
        }
    }
}

impl Display for MetadataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataError::WrongAttributeType { attribute, expected } => {
                write!(f, "Metadata attribute '{attribute}' is not of type '{expected}'!")
            }
            MetadataError::UnknownAttribute { attribute } => {
                write!(f, "Metadata attribute '{attribute}' is unknown!")
            }
        }
    }
}

impl Display for FilterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterError::TestSuiteNotFound { name } => {
                write!(f, "Test suite '{name}' not found!")
            }
            FilterError::TestCaseNotFound { name } => {
                write!(f, "TestCase '{name}' not found.")
            }
            FilterError::TestNotFound { name } => {
                write!(f, "Test '{name}' not found.")
            }
        }
    }
}

impl Display for ParameterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            cause @ ParameterError::IllegalParameterName(_) => {
                write!(f, "{cause}")
            }
        }
    }
}

impl Display for PythonReflectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PythonReflectionError::NoSuchAttribute { attribute_name } => write!(f, "No such attribute: {attribute_name}"),
            PythonReflectionError::AttributeNotWritable { owner_name, attribute_name } => write!(f, "Cannot set the attribute '{attribute_name}' on '{owner_name}'."),
            PythonReflectionError::Downcast { source_type, target_type } => write!(f, "Type '{source_type}' cannot be cast to type '{target_type}'."),
        }
    }
}

impl Display for PythonRuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Display for RunError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let identifier = &self.identifier;
        match &self.kind {
            RunErrorKind::FailedEventEmission { message } => {
                write!(f, "Run failed for '{identifier}', due to an event-emitting error: {message}")
            }
            RunErrorKind::PythonReflectionError { cause } => {
                write!(f, "Run failed for '{identifier}', due to a Python reflection error: {cause}")
            }
        }
    }
}

impl Display for RuntimeInstantiationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
