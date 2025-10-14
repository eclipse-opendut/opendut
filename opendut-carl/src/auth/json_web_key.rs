use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::auth::validation::ValidationError;

#[derive(Debug, Clone)]
pub struct JwkCacheValue {
    pub jwk: JsonWebKey,
    pub last_cached: i64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsonWebKey {
    pub alg: String,
    pub kty: String,
    pub r#use: String,
    #[serde(rename = "n")]
    pub modulus: String,
    #[serde(rename = "e")]
    pub exponent: String,
    #[serde(rename = "kid")]
    pub key_identifier: String,  // kid or key id is the id of the public certificate (of the issuer/identity provider)
    pub x5t: String,
    pub x5c: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OidcJsonWebKeySet {
    pub(crate) keys: Vec<JsonWebKey>,
}

impl OidcJsonWebKeySet {
    pub fn parse(json_web_key: &str) -> Result<BTreeMap<String, JsonWebKey>, ValidationError> {
        let json_web_key_set = serde_json::from_str::<OidcJsonWebKeySet>(json_web_key)
            .map_err(|cause| ValidationError::Configuration(format!("Failed to parse json: {cause}")))?;
        Ok(json_web_key_set.keys.into_iter().map(|jwk| {
            (jwk.key_identifier.clone(), jwk)
        }).collect::<BTreeMap<_, _>>())
    }
}
