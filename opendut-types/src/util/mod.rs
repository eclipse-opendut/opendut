pub mod net;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hostname(pub String);

impl From<String> for Hostname {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&str> for Hostname {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Port(pub u16);

impl From<u16> for Port {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl ToString for Port {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

pub fn valid_characters_in_name(c: &char) -> bool {
    c.is_ascii_alphanumeric() || c.eq(&'-') || c.eq(&'_')
}

pub fn invalid_start_and_end_of_a_name(string: &str) -> bool {
    const INVALID_BEGIN_END_CHARS: [char; 2] = ['_', '-'];
    INVALID_BEGIN_END_CHARS
        .iter()
        .any(|&char| string.starts_with(char) || string.ends_with(char))
}

pub fn invalid_start_and_end_of_location(string: &str) -> bool {
    const INVALID_BEGIN_END_CHARS: [char; 6] = ['_', '-', '/', ' ', '.', ','];
    INVALID_BEGIN_END_CHARS
        .iter()
        .any(|&char| string.starts_with(char) || string.ends_with(char))
}

pub fn valid_characters_in_location(c: &char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/' | ' ' | '.' | ',' | '(' | ')')
}
