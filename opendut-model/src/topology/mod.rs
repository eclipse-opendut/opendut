use std::fmt;
use std::ops::Not;
use std::str::FromStr;

use crate::util::net::NetworkInterfaceId;
use serde::{Deserialize, Serialize};
use crate::create_id_type;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Topology {
    pub devices: Vec<DeviceDescriptor>,
}

impl Topology {
    pub fn new(devices: Vec<DeviceDescriptor>) -> Self {
        Self { devices }
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceDescriptor {
    pub id: DeviceId,
    pub name: DeviceName,
    pub description: Option<DeviceDescription>,
    pub interface: NetworkInterfaceId,
    pub tags: Vec<DeviceTag>,
}


create_id_type!(DeviceId);


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceName(pub(crate) String);

impl DeviceName {
    pub const MIN_LENGTH: usize = 1;
    pub const MAX_LENGTH: usize = 64;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalDeviceName {
    #[error("Device name '{value}' is too short. Expected at least {expected} characters, got {actual}.")]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "Device name '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("Device name '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("Device name '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<DeviceName> for String {
    fn from(value: DeviceName) -> Self {
        value.0
    }
}

impl TryFrom<String> for DeviceName {
    type Error = IllegalDeviceName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalDeviceName::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalDeviceName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalDeviceName::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_name(&c).not())
        {
            Err(IllegalDeviceName::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for DeviceName {
    type Error = IllegalDeviceName;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        DeviceName::try_from(value.to_owned())
    }
}

impl FromStr for DeviceName {
    type Err = IllegalDeviceName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        DeviceName::try_from(value)
    }
}

impl fmt::Display for DeviceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, Eq, Default, PartialEq, Serialize, Deserialize)]
pub struct DeviceDescription(pub(crate) String);

impl DeviceDescription {
    pub const MAX_LENGTH: usize = 280;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalDeviceDescription {
    #[error(
    "Device description '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
}

impl From<DeviceDescription> for String {
    fn from(value: DeviceDescription) -> Self {
        value.0
    }
}

impl TryFrom<String> for DeviceDescription {
    type Error = IllegalDeviceDescription;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length > Self::MAX_LENGTH {
            Err(IllegalDeviceDescription::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for DeviceDescription {
    type Error = IllegalDeviceDescription;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        DeviceDescription::try_from(value.to_owned())
    }
}

impl fmt::Display for DeviceDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DeviceDescription {
    type Err = IllegalDeviceDescription;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DeviceTag(pub(crate) String);

impl DeviceTag {
    pub const MAX_LENGTH: usize = 64;

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalDeviceTag {
    #[error(
        "Device tag '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
}

impl From<DeviceTag> for String {
    fn from(value: DeviceTag) -> Self {
        value.0
    }
}

impl TryFrom<String> for DeviceTag {
    type Error = IllegalDeviceTag;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length > Self::MAX_LENGTH {
            Err(IllegalDeviceTag::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for DeviceTag {
    type Error = IllegalDeviceTag;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        DeviceTag::try_from(value.to_owned())
    }
}

impl fmt::Display for DeviceTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for DeviceTag {
    type Err = IllegalDeviceTag;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}
