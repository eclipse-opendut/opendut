use std::fmt;

use serde::{Deserialize, Serialize};
use crate::util::net::NetworkInterfaceName;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Topology {
    pub devices: Vec<DeviceDescriptor>,
}

impl Topology {
    pub fn new(devices: Vec<DeviceDescriptor>) -> Self {
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
pub struct DeviceDescriptor {
    pub id: DeviceId,
    pub name: String,
    pub description: String,
    pub interface: NetworkInterfaceName,
    pub tags: Vec<String>,
}
