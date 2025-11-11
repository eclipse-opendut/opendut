use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::Not;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::create_id_type;

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
pub struct Certificate(pub pem::Pem);
impl Certificate {
    pub fn encode_as_string(&self) -> String {
        let encode_config = pem::EncodeConfig::default()
            .set_line_ending(pem::LineEnding::LF); //use LF, because `reqwest` fails loading certificate with CRLF with "malformedframing" error

        pem::encode_config(&self.0, encode_config)
    }
}
impl FromStr for Certificate {
    type Err = pem::PemError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        pem::Pem::from_str(value)
            .map(Certificate)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct CanSamplePoint {
    sample_point_times_1000: u32
}
impl CanSamplePoint {
    pub fn sample_point_times_1000(&self) -> u32 {
        self.sample_point_times_1000
    }

    pub fn sample_point(&self) -> f32 {
        (self.sample_point_times_1000 as f32)/1000f32
    }
}

impl TryFrom<f32> for CanSamplePoint {
    type Error = CanSamplePointError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        let sample_point_times_1000 = (value * 1000.0).round() as u32;
        if (0..1000).contains(&sample_point_times_1000) {
            Ok(CanSamplePoint { sample_point_times_1000 })
        } else {
            Err(CanSamplePointError::OutOfRangeFloat { value: value.to_string() })
        }
    }
}

impl TryFrom<u32> for CanSamplePoint {
    type Error = CanSamplePointError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (0..1000).contains(&value) {
            Ok(CanSamplePoint { sample_point_times_1000: value })
        } else {
            Err(CanSamplePointError::OutOfRangeInt { value: value.to_string() })
        }
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
#[serde(rename_all="kebab-case")]
pub enum NetworkInterfaceConfiguration {
    Ethernet,
    Can {
        /// CAN 2.0 Bitrate in Baud
        bitrate: u32,
        /// CAN 2.0 Sample Point between 0.0 and 1.0
        sample_point: CanSamplePoint,
        /// Whether CAN FD should be used
        fd: bool,
        /// CAN FD bitrate in Baud
        data_bitrate: u32,
        /// CAN FD Sample Point between 0.0 and 1.0
        data_sample_point: CanSamplePoint,
    },
    Vcan,
}
impl NetworkInterfaceConfiguration {
    /// Interface is either CAN or VCAN.
    pub fn is_can_like(&self) -> bool {
        matches!(self, Self::Can { .. } | Self::Vcan)
    }
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
            NetworkInterfaceConfiguration::Vcan => write!(f, "VCAN"),
        }
        
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub struct NetworkInterfaceDescriptor {
    pub id: NetworkInterfaceId,
    pub name: NetworkInterfaceName,
    pub configuration: NetworkInterfaceConfiguration,
}
impl fmt::Display for NetworkInterfaceDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.configuration)
    }
}


create_id_type!(NetworkInterfaceId);


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientId(pub String);

impl ClientId {
    pub const MIN_LENGTH: usize = 8;
    pub const MAX_LENGTH: usize = 64;
    pub fn value(self) -> String {
        self.0
    }
}


#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ClientSecret(pub String);
impl Debug for ClientSecret {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("ClientSecret([redacted])")
    }
}

impl ClientSecret {
    pub const MIN_LENGTH: usize = 20;
    pub const MAX_LENGTH: usize = 512;
    pub fn value(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OAuthScope(pub String);

impl OAuthScope {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 48;
    pub fn value(self) -> String {
        self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalOAuthScope {
    #[error(
    "OAuthScope '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
    "OAuthScope '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("OAuthScope '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("OAuthScope '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<OAuthScope> for String {
    fn from(value: OAuthScope) -> Self { value.0 }
}

impl From<&str> for OAuthScope {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}


impl TryFrom<String> for OAuthScope {
    type Error = IllegalOAuthScope;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalOAuthScope::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalOAuthScope::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalOAuthScope::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_name(&c).not())
        {
            Err(IllegalOAuthScope::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}


#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalClientId {
    #[error(
    "Client id '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
    "Client id '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("Client id '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("Client id '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<ClientId> for String {
    fn from(value: ClientId) -> Self { value.0 }
}

impl From<&str> for ClientId {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}

impl TryFrom<String> for ClientId {
    type Error = IllegalClientId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalClientId::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalClientId::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalClientId::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_name(&c).not())
        {
            Err(IllegalClientId::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalClientSecret {
    #[error(
    "Client secret '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
    "Client secret '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
}

impl From<ClientSecret> for String {
    fn from(value: ClientSecret) -> Self { value.0 }
}

impl From<&str> for ClientSecret {
    fn from(value: &str) -> Self {
        Self(String::from(value))
    }
}

impl TryFrom<String> for ClientSecret {
    type Error = IllegalClientSecret;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalClientSecret::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalClientSecret::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else {
            Ok(Self(value))
        }
    }
}


#[derive(Debug, Clone)]
pub struct ClientCredentials {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AuthConfig {
    Enabled {
        issuer_url: Url,
        client_id: ClientId,
        client_secret: ClientSecret,
        scopes: Vec<OAuthScope>,
    },
    Disabled,
}

impl AuthConfig {
     pub fn from_credentials(issuer_url: Url, client_credentials: ClientCredentials) -> Self {

        Self::Enabled {
            issuer_url,
            client_id: client_credentials.client_id,
            client_secret: client_credentials.client_secret,
            scopes: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use googletest::assert_that;
    use googletest::matchers::eq;
    use url::Url;

    use crate::util::net::{AuthConfig, ClientCredentials, ClientId, ClientSecret, OAuthScope};

    #[test]
    pub fn test_create_auth_config() {
        let client_id = ClientId("test".to_string());
        let client_secret = ClientSecret("foobar".to_string());
        let client_credentials = ClientCredentials { client_id: client_id.clone(), client_secret: client_secret.clone() };
        let expected_scopes: Vec<OAuthScope> = vec![];
        let issuer_url = Url::parse("https://some-address-idk.com").unwrap();
        let auth_config = AuthConfig::from_credentials(issuer_url.clone(), client_credentials);

        assert_that!(auth_config, eq(&AuthConfig::Enabled {
            issuer_url,
            client_id,
            client_secret,
            scopes: expected_scopes,
        }));
    }
}
