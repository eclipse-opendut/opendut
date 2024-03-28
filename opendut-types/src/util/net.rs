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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct CanSamplePoint {
    sample_point_times_1000: i32
}

impl TryFrom<f32> for CanSamplePoint {
    type Error = CanSamplePointError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let sample_point_times_1000 = (value * 1000.0).round() as i32;
        if (0..1000).contains(&sample_point_times_1000) {
            Ok(CanSamplePoint{sample_point_times_1000})
        } else {
            Err(CanSamplePointError::OutOfRangeFloat { value: value.to_string() })
        }
    }
}

impl TryFrom<i32> for CanSamplePoint {
    type Error = CanSamplePointError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if (0..1000).contains(&value) {
            Ok(CanSamplePoint{sample_point_times_1000: value})
        } else {
            Err(CanSamplePointError::OutOfRangeInt { value: value.to_string() })
        }
    }
}

impl From<CanSamplePoint> for i32 {
    fn from(value: CanSamplePoint) -> Self {
        value.sample_point_times_1000
    }
}

impl fmt::Display for CanSamplePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0.{:0>3}", self.sample_point_times_1000)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CanSamplePointError {
    #[error("Sample point must be in the range [0.000, 0.999] but is {value}")]
    OutOfRangeFloat { value: String },
    #[error("Integer to create sample point from must be in the range [0, 999] but is {value}")]
    OutOfRangeInt { value: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum NetworkInterfaceConfiguration {
    Ethernet,
    Can {
        bitrate: i32,
        sample_point: CanSamplePoint,
        fd: bool,
        data_bitrate: i32,
        data_sample_point: CanSamplePoint,
    },
}

impl fmt::Display for NetworkInterfaceConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkInterfaceConfiguration::Ethernet => write!(f, "Ethernet"),
            NetworkInterfaceConfiguration::Can { 
                bitrate, 
                sample_point, 
                fd, 
                data_bitrate, 
                data_sample_point 
            } => write!(f, "CAN [bitrate: {bitrate}, sample point: {sample_point}, fd: {fd}, data bitrate: {data_bitrate}, data sample point: {data_sample_point}]"),
        }
        
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct NetworkInterfaceDescriptor {
    pub name: NetworkInterfaceName,
    pub configuration: NetworkInterfaceConfiguration,
}

impl fmt::Display for NetworkInterfaceDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.configuration)
        
    }
}

