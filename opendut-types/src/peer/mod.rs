use std::fmt;
use std::ops::Not;
use std::str::FromStr;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::peer::executor::ExecutorDescriptors;
use crate::topology::{DeviceDescriptor, Topology};
use crate::util::net::{AuthConfig, Certificate, NetworkInterfaceDescriptor, NetworkInterfaceName};
use crate::vpn::VpnPeerConfiguration;

pub mod state;
pub mod executor;
pub mod configuration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerId { pub uuid: Uuid }

impl PeerId {
    pub fn random() -> Self {
        Self { uuid: Uuid::new_v4() }
    }
}

impl From<Uuid> for PeerId {
    fn from(uuid: Uuid) -> Self {
        Self { uuid }
    }
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("Illegal PeerId: {value}")]
pub struct IllegalPeerId {
    pub value: String,
}

impl TryFrom<&str> for PeerId {
    type Error = IllegalPeerId;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value).map(|uuid| Self { uuid }).map_err(|_| IllegalPeerId {
            value: String::from(value),
        })
    }
}

impl TryFrom<String> for PeerId {
    type Error = IllegalPeerId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PeerId::try_from(value.as_str())
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uuid)
    }
}

impl FromStr for PeerId {
    type Err = IllegalPeerId;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerName(pub(crate) String);

impl PeerName {
    pub const MIN_LENGTH: usize = 4;
    pub const MAX_LENGTH: usize = 64;

    pub fn value(self) -> String {
        self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalPeerName {
    #[error(
        "Peer name '{value}' is too short. Expected at least {expected} characters, got {actual}."
    )]
    TooShort {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error(
        "Peer name '{value}' is too long. Expected at most {expected} characters, got {actual}."
    )]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("Peer name '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("Peer name '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<PeerName> for String {
    fn from(value: PeerName) -> Self {
        value.0
    }
}

impl TryFrom<String> for PeerName {
    type Error = IllegalPeerName;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length < Self::MIN_LENGTH {
            Err(IllegalPeerName::TooShort {
                value,
                expected: Self::MIN_LENGTH,
                actual: length,
            })
        } else if length > Self::MAX_LENGTH {
            Err(IllegalPeerName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_a_name(&value) {
            Err(IllegalPeerName::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_name(&c).not())
        {
            Err(IllegalPeerName::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for PeerName {
    type Error = IllegalPeerName;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        PeerName::try_from(value.to_owned())
    }
}

impl fmt::Display for PeerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PeerName {
    type Err = IllegalPeerName;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerLocation(pub(crate) String);

impl PeerLocation {
    pub const MAX_LENGTH: usize = 64;

    pub fn value(self) -> String {
        self.0
    }
}

#[derive(thiserror::Error, Clone, Debug)]
pub enum IllegalLocation {
    #[error("Peer location '{value}' is too long. Expected at most {expected} characters, got {actual}.")]
    TooLong {
        value: String,
        expected: usize,
        actual: usize,
    },
    #[error("Peer location '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
    #[error("Peer location '{value}' contains invalid start or end characters.")]
    InvalidStartEndCharacter { value: String },
}

impl From<PeerLocation> for String {
    fn from(value: PeerLocation) -> Self {
        value.0
    }
}

impl TryFrom<String> for PeerLocation {
    type Error = IllegalLocation;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let length = value.len();
        if length > Self::MAX_LENGTH {
            Err(IllegalLocation::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::invalid_start_and_end_of_location(&value) {
            Err(IllegalLocation::InvalidStartEndCharacter { value })
        } else if value
            .chars()
            .any(|c| crate::util::valid_characters_in_location(&c).not())
        {
            Err(IllegalLocation::InvalidCharacter { value })
        } else {
            Ok(Self(value))
        }
    }
}

impl TryFrom<&str> for PeerLocation {
    type Error = IllegalLocation;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        crate::peer::PeerLocation::try_from(value.to_owned())
    }
}

impl fmt::Display for PeerLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for PeerLocation {
    type Err = IllegalLocation;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerNetworkDescriptor {
    pub interfaces: Vec<NetworkInterfaceDescriptor>,
    pub bridge_name: Option<NetworkInterfaceName>,
}

impl PeerNetworkDescriptor {
    pub fn new(interfaces: Vec<NetworkInterfaceDescriptor>, bridge_name: Option<NetworkInterfaceName>) -> Self {
        Self { interfaces, bridge_name}
    }
    pub fn interfaces_zipped_with_devices(&self, devices: &[DeviceDescriptor]) -> Vec<(NetworkInterfaceDescriptor, DeviceDescriptor)> {
        devices.iter()
            .cloned()
            .map(|device| {
                let interface = self.interfaces.iter()
                    .find(|interface_descriptor| interface_descriptor.id == device.interface)
                    .expect("Should always have a NetworkInterfaceDescriptor for device's network interface on a PeerDescriptor.")
                    .clone();
                (interface, device)
            }).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerDescriptor {
    pub id: PeerId,
    pub name: PeerName,
    pub location: Option<PeerLocation>,
    pub network: PeerNetworkDescriptor,
    pub topology: Topology,
    pub executors: ExecutorDescriptors,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerSetup {
    pub id: PeerId,
    pub carl: Url,
    pub ca: Certificate,
    pub auth_config: AuthConfig,
    pub vpn: VpnPeerConfiguration,
}

impl PeerSetup {
    pub fn encode(&self) -> Result<String, PeerSetupEncodeError> {
        let json = serde_json::to_string(self).map_err(|cause| PeerSetupEncodeError {
            details: format!("Serialization failed due to: {}", cause),
        })?;

        let compressed = {
            let mut buffer = Vec::new();
            crate::util::brotli::compress(&mut buffer, json.as_bytes())
                .map_err(|cause| PeerSetupEncodeError {
                    details: format!("Compression failed due to: {}", cause),
                })?;
            buffer
        };

        let encoded = BASE64_URL_SAFE.encode(compressed);

        Ok(encoded)
    }

    pub fn decode(encoded: &str) -> Result<Self, PeerSetupDecodeError> {
        let compressed = BASE64_URL_SAFE
            .decode(encoded.as_bytes())
            .map_err(|cause| PeerSetupDecodeError {
                details: format!("Base64 decoding failed due to: {}", cause),
            })?;

        let json = {
            let mut buffer = Vec::new();
            crate::util::brotli::decompress(&mut buffer, compressed.as_slice())
                .map_err(|cause| PeerSetupDecodeError {
                    details: format!("Decompression failed due to: {}", cause),
                })?;
            buffer
        };

        let decoded = serde_json::from_slice(&json).map_err(|cause| PeerSetupDecodeError {
            details: format!("Deserialization failed due to: {}", cause),
        })?;

        Ok(decoded)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("PeerSetup could not be encoded. {details}")]
pub struct PeerSetupEncodeError {
    details: String,
}

#[derive(thiserror::Error, Debug)]
#[error("PeerSetup could not be decoded. {details}")]
pub struct PeerSetupDecodeError {
    details: String,
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use googletest::prelude::*;
    use pem::Pem;
    use uuid::Uuid;

    use crate::vpn::netbird::SetupKey;

    use super::*;
    use crate::util::net::{ClientId, ClientSecret, OAuthScope};

    #[test]
    fn A_PeerSetup_should_be_encodable() -> Result<()> {
        let setup = PeerSetup {
            id: PeerId::try_from("01bf3f8c-cc7c-4114-9520-91bce71dcead").unwrap(),
            carl: Url::parse("https://carl.opendut.local")?,
            ca: Certificate(Pem::new("Test Tag".to_string(), vec![])),
            auth_config: AuthConfig::Enabled {
                client_id: ClientId::from("client_id"),
                client_secret: ClientSecret::from("my-secure!-random-string-with-at-least-x-chars%"),
                scopes: vec![OAuthScope::from("manage-realm")],
                issuer_url: Url::parse("https://keycloak/realms/opendut/").unwrap(),
            },
            vpn: VpnPeerConfiguration::Netbird {
                management_url: Url::parse("https://netbird.opendut.local/api")?,
                setup_key: SetupKey::from(Uuid::parse_str("d79c202f-bbbf-4997-844e-678f27606e1c")?),
            },
        };

        let encoded = setup.encode()?;
        assert_that!(encoded, eq("F8sBABwHzrk8VikIvG691CJ7W1Stsq0oZqoIe8UvyoH_PRFpnb4ullM3wV7g8UFoTRxt-svCaEwMLLSAA7DMXeWhPNSyiLPbPh7iH2lyQsi01GXIkGU-g5HSILFKIJFpVniZZ0XMabK49sS7TWmybMacHztb1vGBJFsITgY9fIGLFdBn_YygcXHdu6CJx17_kdGTXHZEVbbx5_p3wJh2xZP1U-w47d2wcUQUC-fE_u3xqINxDkbE8MD44QUiPdzFwtIZcYdvd7jxsT3zrsw9ho2cozsAmNJrznkEXI_FHmRUPZj8BvjFuCsrVnFuCovbd3TFQ-Q-yZRQJVKBGx9MkngEYwo4H0rlnXCFzKgC"));

        let decoded = PeerSetup::decode(&encoded)?;
        assert_that!(decoded, eq(&setup));

        Ok(())
    }

    #[test]
    fn A_PeerName_should_contain_valid_characters() -> Result<()> {
        let peer_name =
            PeerName::try_from("asd123".to_string()).expect("Failed to create peer name");
        assert_eq!(peer_name.0, "asd123");
        Ok(())
    }

    #[test]
    fn A_PeerName_may_contain_a_hyphen() -> Result<()> {
        let peer_name =
            PeerName::try_from("asd-123".to_string()).expect("Failed to create peer name");
        assert_eq!(peer_name.0, "asd-123");
        Ok(())
    }

    #[test]
    fn A_PeerName_may_contain_an_underscore() -> Result<()> {
        let peer_name =
            PeerName::try_from("asd-123".to_string()).expect("Failed to create peer name");
        assert_eq!(peer_name.0, "asd-123");
        Ok(())
    }

    #[test]
    fn A_PeerName_should_not_start_with_a_hyphen() -> Result<()> {
        let _ = PeerName::try_from("-123".to_string()).is_err();
        Ok(())
    }
}
