use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Topology {
    pub devices: Vec<Device>,
}

impl Topology {
    pub fn new(devices: Vec<Device>) -> Self {
        Self { devices }
    }
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

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InterfaceName { name: String }
impl InterfaceName {
    pub const MAX_LENGTH: usize = 15;
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
            Err(InterfaceNameError::Empty)
        } else if value.len() > Self::MAX_LENGTH {
            Err(InterfaceNameError::TooLong { value, max: Self::MAX_LENGTH })
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

#[derive(thiserror::Error, Debug)]
pub enum InterfaceNameError {
    #[error("Name for network interface may not be empty!")]
    Empty,
    #[error("Due to operating system limitations, the name for network interfaces may not be longer than {max} characters!")]
    TooLong { value: String, max: usize }
}
