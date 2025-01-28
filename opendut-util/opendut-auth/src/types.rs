use std::fmt::Debug;
use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum Audience {
    SingleAudience(String),
    MultipleAudiences(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Audience
    #[serde(rename = "aud")]
    pub audience: Audience,
    /// Issued at (as UTC timestamp)
    #[serde(rename = "iat")]
    pub issued_at: usize,
    /// Issuer
    #[serde(rename = "iss")]
    pub issuer: String,
    /// Expiration time (as UTC timestamp)
    #[serde(rename = "exp")]
    pub expiration_utc: usize,
    /// Subject (whom token refers to)
    #[serde(rename = "sub")]
    pub subject: String,
    // Name of the user
    pub name: String,
    // Email address of the user
    pub email: String,
    // Username of the user
    pub preferred_username: String,
    #[serde(flatten)]
    pub additional_claims: MyAdditionalClaims
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MyAdditionalClaims {
    /// Roles the user belongs to (custom claim)
    #[serde(default = "MyAdditionalClaims::empty_vector")]
    pub roles: Vec<String>,
    /// Groups of the user (custom claim) may be omitted by identity provider, so we need a default value
    #[serde(default = "MyAdditionalClaims::empty_vector")]
    pub groups: Vec<String>,
}

impl MyAdditionalClaims {
    fn empty_vector() -> Vec<String> { Vec::new() }
}

cfg_if! {
    if #[cfg(feature = "registration_client")] {
        use openidconnect::AdditionalClaims;
        impl AdditionalClaims for MyAdditionalClaims {}
    }
}
