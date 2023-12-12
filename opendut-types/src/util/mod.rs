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
