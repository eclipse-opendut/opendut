use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Topology {
    pub devices: Vec<Device>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(pub uuid::Uuid);

impl DeviceId {

    pub const NIL: Self = Self(uuid::Uuid::from_bytes([0; 16]));

    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for DeviceId {
    fn default() -> Self {
        Self::NIL
    }
}

impl From<uuid::Uuid> for DeviceId {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    pub description: String,
    pub location: String,
    pub interface: InterfaceName,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InterfaceName { name: String }
impl InterfaceName {
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl fmt::Display for InterfaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<String> for InterfaceName {
    type Error = InterfaceNameError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(InterfaceNameError { message: format!("Interface name may not be empty.") })
        } else if value.len() > 15 {
            Err(InterfaceNameError { message: format!("Interface name '{value}' is longer than 15 characters. This is not supported by Linux.") })
        } else {
            Ok(Self { name: value })
        }
    }
}

impl TryFrom<&str> for InterfaceName {
    type Error = InterfaceNameError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(String::from(value))
    }
}

impl std::str::FromStr for InterfaceName {
    type Err = InterfaceNameError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(String::from(value))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct InterfaceNameError { message: String }
