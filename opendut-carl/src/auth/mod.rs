mod validation;
pub(crate) mod json_web_key;
pub(crate) mod grpc_auth_layer;

use openidconnect::core::CoreGenderClaim;
use openidconnect::{AdditionalClaims, IdTokenClaims};
use serde::{Deserialize, Serialize};

pub type Claims<AC> = IdTokenClaims<AC, CoreGenderClaim>;

#[allow(unused)] //TODO implement authorization
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub name: String,
    pub claims: Claims<MyAdditionalClaims>,
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

impl AdditionalClaims for MyAdditionalClaims {}
