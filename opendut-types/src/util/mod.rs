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

pub fn valid_characters_in_name(c: &char) -> bool {
    c.is_ascii_alphanumeric() || c.eq(&'-') || c.eq(&'_')
}

pub fn valid_start_and_end_of_a_name(string: &str) -> bool {
    let invalid_begin_end_chars = vec!['_', '-'];
    for char in invalid_begin_end_chars {
        if string.starts_with(char) || string.ends_with(char) {
            return false
        }
    }
    true
}
