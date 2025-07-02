use std::fmt;
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;
use crate::util::net::{AuthConfig, Certificate};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CleoId(pub Uuid);

impl CleoId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for CleoId{
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(thiserror::Error, Clone, Debug)]
#[error("Illegal CleoId: {value}")]
pub struct IllegalCleoId {
    pub value: String,
}

impl TryFrom<&str> for CleoId {
    type Error = IllegalCleoId;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value).map(Self).map_err(|_| IllegalCleoId {
            value: String::from(value),
        })
    }
}

impl TryFrom<String> for CleoId {
    type Error = IllegalCleoId;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CleoId::try_from(value.as_str())
    }
}

impl fmt::Display for CleoId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CleoSetup {
    pub id: CleoId,
    pub carl: Url,
    pub ca: Certificate,
    pub auth_config: AuthConfig,
}

impl CleoSetup {
    pub fn encode(&self) -> Result<String, CleoSetupEncodeError> {
        let json = serde_json::to_string(self).map_err(|cause| CleoSetupEncodeError {
            details: format!("Serialization failed due to: {cause}"),
        })?;

        let compressed = {
            let mut buffer = Vec::new();
            crate::util::brotli::compress(&mut buffer, json.as_bytes())
                .map_err(|cause| CleoSetupEncodeError {
                    details: format!("Compression failed due to: {cause}"),
                })?;
            buffer
        };

        let encoded = BASE64_URL_SAFE.encode(compressed);

        Ok(encoded)
    }

    pub fn decode(encoded: &str) -> Result<Self, CleoSetupDecodeError> {
        let compressed = BASE64_URL_SAFE
            .decode(encoded.as_bytes())
            .map_err(|cause| CleoSetupDecodeError {
                details: format!("Base64 decoding failed due to: {cause}"),
            })?;

        let json = {
            let mut buffer = Vec::new();
            crate::util::brotli::decompress(&mut buffer, compressed.as_slice())
                .map_err(|cause| CleoSetupDecodeError {
                    details: format!("Decompression failed due to: {cause}"),
                })?;
            buffer
        };

        let decoded = serde_json::from_slice(&json).map_err(|cause| CleoSetupDecodeError {
            details: format!("Deserialization failed due to: {cause}"),
        })?;

        Ok(decoded)
    }
}

#[derive(thiserror::Error, Debug)]
#[error("CleoSetup could not be encoded. {details}")]
pub struct CleoSetupEncodeError {
    details: String,
}

#[derive(thiserror::Error, Debug)]
#[error("CleoSetup could not be decoded. {details}")]
pub struct CleoSetupDecodeError {
    details: String,
}
