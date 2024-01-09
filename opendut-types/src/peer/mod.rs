use std::fmt;
use std::io::{Read, Write};
use std::ops::Not;

use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use crate::topology::Topology;

use crate::vpn::VpnPeerConfig;

pub mod state;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerId(pub Uuid);

impl PeerId {

    pub const NIL: Self = Self(Uuid::from_bytes([0; 16]));

    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PeerId {
    fn default() -> Self {
        Self::NIL
    }
}

impl From<Uuid> for PeerId {
    fn from(value: Uuid) -> Self {
        Self(value)
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
        Uuid::parse_str(value)
            .map(Self)
            .map_err(|_| IllegalPeerId { value: String::from(value) })
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
        write!(f, "{}", self.0)
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
    #[error("Peer name '{value}' is too short. Expected at least {expected} characters, got {actual}.")]
    TooShort { value: String, expected: usize, actual: usize },
    #[error("Peer name '{value}' is too long. Expected at most {expected} characters, got {actual}.")]
    TooLong { value: String, expected: usize, actual: usize },
    #[error("Peer name '{value}' contains invalid characters.")]
    InvalidCharacter { value: String },
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
        }
        else if length > Self::MAX_LENGTH {
            Err(IllegalPeerName::TooLong {
                value,
                expected: Self::MAX_LENGTH,
                actual: length,
            })
        } else if crate::util::valid_start_and_end_of_a_name(&value).not()
            || value.chars().any(|c| crate::util::valid_characters_in_name(&c).not()) {
                Err(IllegalPeerName::InvalidCharacter {
                    value
                })
        }
        else {
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerDescriptor {
    pub id: PeerId,
    pub name: PeerName,
    pub topology: Topology,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeerSetup {
    pub id: PeerId,
    pub carl: Url,
    pub vpn: VpnPeerConfig,
}

impl PeerSetup {

    pub fn encode(&self) -> Result<String, PeerSetupEncodeError> {

        let json = serde_json::to_string(self)
            .map_err(|cause| PeerSetupEncodeError { details: format!("Serialization failed due to: {}", cause) })?;

        let compressed = {
            let mut buffer = Vec::new();
            brotli::CompressorReader::new(json.as_bytes(), 4096, 11, 20)
                .read_to_end(&mut buffer)
                .map_err(|cause| PeerSetupEncodeError { details: format!("Compression failed due to: {}", cause) } )?;
            buffer
        };

        let encoded = BASE64_URL_SAFE.encode(compressed);

        Ok(encoded)
    }

    pub fn decode(encoded: &str) -> Result<Self, PeerSetupDecodeError> {

        let compressed = BASE64_URL_SAFE.decode(encoded.as_bytes())
            .map_err(|cause| PeerSetupDecodeError { details: format!("Base64 decoding failed due to: {}", cause) } )?;

        let json = {
            let mut buffer = Vec::new();
            brotli::DecompressorWriter::new(&mut buffer, 4096)
                .write_all(compressed.as_slice())
                .map_err(|cause| PeerSetupDecodeError { details: format!("Decompression failed due to: {}", cause) } )?;
            buffer
        };

        let decoded = serde_json::from_slice(&json)
            .map_err(|cause| PeerSetupDecodeError { details: format!("Deserialization failed due to: {}", cause) })?;

        Ok(decoded)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("PeerSetup could not be encoded. {details}")]
pub struct PeerSetupEncodeError { details: String }

#[derive(thiserror::Error, Debug)]
#[error("PeerSetup could not be decoded. {details}")]
pub struct PeerSetupDecodeError { details: String }

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use googletest::prelude::*;
    use uuid::Uuid;

    use crate::vpn::netbird::SetupKey;

    use super::*;

    #[test]
    fn A_PeerSetup_should_be_encodable() -> Result<()> {

        let setup = PeerSetup {
            id: PeerId::try_from("01bf3f8c-cc7c-4114-9520-91bce71dcead").unwrap(),
            carl: Url::parse("https://carl.opendut.local")?,
            vpn: VpnPeerConfig::Netbird {
                management_url: Url::parse("https://netbird.opendut.local/api")?,
                setup_key: SetupKey::from(Uuid::parse_str("d79c202f-bbbf-4997-844e-678f27606e1c")?),
            }
        };

        let encoded = setup.encode()?;
        assert_that!(encoded, eq("F80A4BwHzrk8ZiEIvG69VCJ7i3MqH11CJ0PdEVOHpU8nB-z_LoHFkbTAUo3iACy3MfC5PN7KI78f7QUIkVZegy1zZ1tiLHbWFOyMxNqjsKJAGB8n4GFTwNypkPfL5Q0ejM_tArroq19ASuv2va2BSo63PVnZjT3qH5A-OYW0LBH5UObsdpSitvWxUm-haWT4AQ=="));

        let decoded = PeerSetup::decode(&encoded)?;
        assert_that!(decoded, eq(setup));

        Ok(())
    }

    #[test]
    fn A_PeerName_should_contain_valid_characters() -> Result<()> {
        let peer_name = PeerName::try_from("asd123".to_string()).expect("Failed to create peer name");
        assert_eq!(peer_name.0, "asd123");
        Ok(())
    }

    #[test]
    fn A_PeerName_may_contain_a_hyphen() -> Result<()> {
        let peer_name = PeerName::try_from("asd-123".to_string()).expect("Failed to create peer name");
        assert_eq!(peer_name.0, "asd-123");
        Ok(())
    }

    #[test]
    fn A_PeerName_may_contain_an_underscore() -> Result<()> {
        let peer_name = PeerName::try_from("asd-123".to_string()).expect("Failed to create peer name");
        assert_eq!(peer_name.0, "asd-123");
        Ok(())
    }

    #[test]
    fn A_PeerName_should_not_start_with_a_hyphen() -> Result<()> {
        let _ = PeerName::try_from("-123".to_string()).is_err();
        Ok(())
    }
}
