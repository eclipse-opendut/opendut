use std::fmt::{Display, Formatter};
use viper_rt::run::{BindParameterError, IncompleteParameterBindingsError};

#[derive(Debug)]
pub struct ParameterTomlError {
    pub suite: String,
    pub cause: BindParameterError,
}

impl std::error::Error for ParameterTomlError {}

impl Display for ParameterTomlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to load parameters for suite '{}': {}", self.suite, self.cause)
    }
}

#[derive(Debug)]
pub struct IncompleteBindingsError {
    pub suite: String,
    pub cause: IncompleteParameterBindingsError
}

impl std::error::Error for IncompleteBindingsError {}

impl Display for IncompleteBindingsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})",self.cause, self.suite)
    }
}
