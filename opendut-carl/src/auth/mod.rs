mod validation;
pub(crate) mod json_web_key;
pub(crate) mod grpc_auth_layer;
pub mod in_memory_cache;

use openidconnect::core::CoreGenderClaim;
use openidconnect::IdTokenClaims;
use opendut_auth::types::MyAdditionalClaims;

pub type Claims<AC> = IdTokenClaims<AC, CoreGenderClaim>;

#[allow(unused)] //TODO implement authorization
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub name: String,
    pub claims: Claims<MyAdditionalClaims>,
}
