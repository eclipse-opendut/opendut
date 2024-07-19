use serde::{Deserialize, Serialize};

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