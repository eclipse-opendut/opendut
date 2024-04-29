pub mod cluster;

use std::str::FromStr;


#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("Could not parse '{from}' as {to}: {details}")]
pub struct ParseError {
    from: String,
    to: &'static str,
    details: String,
}

impl ParseError {
    pub fn new<To>(from: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: std::any::type_name::<To>(),
            details: details.into(),
        }
    }
}
