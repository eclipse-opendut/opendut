use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Parameter {
    pub name: String,
    pub value: String,
}

impl FromStr for Parameter {
    type Err = ParseError;

    fn from_str(param_string: &str) -> Result<Self, Self::Err> {
        if let Some((name, value)) = param_string.split_once('=') {
            Ok(
                Self {
                    name: String::from(name),
                    value: String::from(value),
                }
            )
        } else {
            Err(
                ParseError {
                    message: format!("Failed to parse '{param_string}' as a parameter."),
                }
            )
        }
    }
}


#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.message)
    }
}
