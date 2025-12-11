use crate::runtime::types::naming::Identifier;
use crate::runtime::types::py::error::PythonReflectionError;

pub type RunResult<T> = Result<T, Box<RunError>>;

/// Error that occurred while executing a test suite.
///
/// Contains an [`Identifier`] to identify the source or context of the error
/// and details in the [`RunErrorKind`] variant.
///
/// # Notes
/// Ensure you account for the `#[non_exhaustive]` nature of this error.
///
#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub struct RunError {
    pub identifier: Box<dyn Identifier>,
    pub kind: RunErrorKind,
}

/// Represents the various kinds of errors that can occur during the execution of a test suite.
///
/// # Note
/// This enum is marked as `#[non_exhaustive]` to allow for future expansion.
///
#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum RunErrorKind {
    FailedEventEmission {
        message: String,
    },
    PythonReflectionError {
        cause: PythonReflectionError,
    },
}

#[allow(dead_code)]
impl RunError {

    pub(crate) fn new(
        identifier: impl Identifier + 'static,
        kind: RunErrorKind,
    ) -> Self {
        Self {
            identifier: Box::new(identifier),
            kind,
        }
    }

    pub(crate) fn new_failed_event_emission_error(
        identifier: impl Identifier + 'static,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            identifier,
            RunErrorKind::FailedEventEmission {
                message: message.into(),
            }
        )
    }

    pub(crate) fn new_python_reflection_error(
        identifier: impl Identifier + 'static,
        cause: PythonReflectionError
    ) -> Self {
        Self::new(
            identifier,
            RunErrorKind::PythonReflectionError {
                cause,
            }
        )
    }
}
