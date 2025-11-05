use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use serde::{Deserialize, Serialize};
use url::Url;
use crate::create_id_type;
use crate::util::net::{AuthConfig, Certificate};


create_id_type!(CleoId);


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
