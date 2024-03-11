use std::fmt;
use pem::Pem;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NetworkInterfaceName { name: String }
impl NetworkInterfaceName {
    pub const MAX_LENGTH: usize = 15;
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

impl fmt::Display for NetworkInterfaceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl TryFrom<String> for NetworkInterfaceName {
    type Error = NetworkInterfaceNameError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            Err(NetworkInterfaceNameError::Empty)
        } else if value.len() > Self::MAX_LENGTH {
            Err(NetworkInterfaceNameError::TooLong { value, max: Self::MAX_LENGTH })
        } else {
            Ok(Self { name: value })
        }
    }
}

impl TryFrom<&str> for NetworkInterfaceName {
    type Error = NetworkInterfaceNameError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(String::from(value))
    }
}

impl std::str::FromStr for NetworkInterfaceName {
    type Err = NetworkInterfaceNameError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(String::from(value))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum NetworkInterfaceNameError {
    #[error("Name for network interface may not be empty!")]
    Empty,
    #[error("Due to operating system limitations, the name for network interfaces may not be longer than {max} characters!")]
    TooLong { value: String, max: usize }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Certificate(pub Pem);
