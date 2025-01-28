use serde::{Deserialize, Serialize};
use url::Url;



#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LeaIdentityProviderConfig {
    pub client_id: String,
    pub issuer_url: Url,
    pub scopes: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct LeaConfig {
    pub carl_url: Url,
    pub idp_config: Option<LeaIdentityProviderConfig>,
}
