use crate::common::TestSuiteIdentifier;
use crate::runtime::source::SourceLoaderError;
use crate::runtime::types::compile::inspect::InspectionError;
use crate::runtime::types::py::error::{PythonReflectionError, PythonRuntimeError};
use crate::runtime::types::source::error::InvalidSourceError;
use crate::source::{Source, SourceLocation};

pub type CompileResult<T> = Result<T, Box<CompilationError>>;

/// Error that occurred while processing a source.
///
/// Contains the name of the source where the error occurred and the specific
/// error details in the [`CompilationErrorKind`] variant.
///
/// # Notes
/// Ensure you account for the `#[non_exhaustive]` nature of this error.
///
#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub struct CompilationError {
    pub identifier: TestSuiteIdentifier,
    pub kind: CompilationErrorKind,
}

/// Represents the various kinds of errors that can occur during the compilation process.
///
/// # Note
/// This enum is marked as `#[non_exhaustive]` to allow for future expansion.
///
#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum CompilationErrorKind {
    FailedEventEmission {
        message: String,
    },
    NoSuitableSourceLoader {
        source_location: SourceLocation,
    },
    FailedLoading {
        source_location: SourceLocation,
        cause: SourceLoaderError,
    },
    InvalidSource {
        cause: InvalidSourceError,
    },
    FailedInspection {
        cause: InspectionError,
    },
    PythonCompilationError {
        details: String,
    },
    PythonReflectionError {
        cause: PythonReflectionError,
    },
    PythonRuntimeError {
        cause: PythonRuntimeError,
    }
}

impl CompilationError {

    pub(crate) fn new(identifier: TestSuiteIdentifier, kind: CompilationErrorKind) -> Self {
        Self { identifier, kind }
    }

    pub(crate) fn new_failed_event_emission_error(
        source: &Source,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            Clone::clone(&source.identifier),
            CompilationErrorKind::FailedEventEmission {
                message: message.into(),
            }
        )
    }

    pub(crate) fn new_source_loading_failure_error(
        source: &Source,
        cause: SourceLoaderError,
    ) -> Self {
        Self::new(
            Clone::clone(&source.identifier),
            CompilationErrorKind::FailedLoading {
                source_location: Clone::clone(&source.location),
                cause
            }
        )
    }

    pub(crate) fn new_invalid_source_error(
        source: &Source,
        cause: InvalidSourceError
    ) -> Self {
        Self::new(
            Clone::clone(&source.identifier),
            CompilationErrorKind::InvalidSource {
                cause
            }
        )
    }

    pub(crate) fn new_inspection_failure_error(
        source: &Source,
        cause: InspectionError
    ) -> Self {
        Self::new(
            Clone::clone(&source.identifier),
            CompilationErrorKind::FailedInspection {
                cause
            }
        )
    }

    pub(crate) fn new_no_suitable_source_loader_error(
        source: &Source
    ) -> Self {
        Self::new(
            Clone::clone(&source.identifier),
            CompilationErrorKind::NoSuitableSourceLoader {
                source_location: Clone::clone(&source.location),
            }
        )
    }

    pub(crate) fn new_python_compilation_error(
        identifier: TestSuiteIdentifier,
        details: impl Into<String>
    ) -> Self {
        Self::new(
            identifier,
            CompilationErrorKind::PythonCompilationError {
                details: details.into(),
            }
        )
    }

    pub(crate) fn new_python_runtime_error(
        identifier: TestSuiteIdentifier,
        cause: PythonRuntimeError
    ) -> Self {
        Self::new(
            identifier,
            CompilationErrorKind::PythonRuntimeError {
                cause,
            }
        )
    }

    pub(crate) fn new_python_reflection_error(
        identifier: TestSuiteIdentifier,
        cause: PythonReflectionError
    ) -> Self {
        Self::new(
            identifier,
            CompilationErrorKind::PythonReflectionError {
                cause,
            }
        )
    }
}
