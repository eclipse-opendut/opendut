use anyhow::anyhow;
use config::Config;
use shadow_rs::formatcp;
use serde::Deserialize;
use url::Url;
use oauth2::{Scope as OAuthScope, ClientId as OAuthClientId, ClientSecret as OAuthClientSecret};

#[derive(Clone, Debug, Deserialize)]
pub struct OidcIdentityProviderConfig {
    pub client_id: OAuthClientId,
    pub client_secret: OAuthClientSecret,
    pub issuer_url: Url,
    pub scopes: Vec<OAuthScope>,
}


const OIDC_CLIENT_CONFIG_PREFIX: &str = "network.oidc.client";
impl OidcIdentityProviderConfig {
    const CLIENT_ID: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.client.id");
    const CLIENT_SECRET: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.client.secret");
    const ISSUER_URL: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.issuer.url");
    const SCOPES: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.scopes");

    pub fn parse_scopes(client_id: &str, raw_scopes: String) -> Vec<OAuthScope> {
        let scopes = raw_scopes.trim_matches('"').split(',').collect::<Vec<_>>();
        for scope in scopes.clone() {
            if !scope.chars().all(|c| c.is_ascii_alphabetic()) {
                panic!("Failed to parse comma-separated OIDC scopes for client_id='{}'. Scopes must only contain ASCII alphabetic characters. Found: {:?}. Parsed as: {:?}", client_id, raw_scopes, scopes);
            }
        }
        scopes.into_iter().map(|scope| OAuthScope::new(scope.to_string())).collect()

    }

}

impl TryFrom<&Config> for OidcIdentityProviderConfig {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> anyhow::Result<Self> {
        let client_id = config.get_string(OidcIdentityProviderConfig::CLIENT_ID)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::CLIENT_ID, error.to_string()))?;
        let client_secret = config.get_string(OidcIdentityProviderConfig::CLIENT_SECRET)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::CLIENT_SECRET, error.to_string()))?;
        let issuer = config.get_string(OidcIdentityProviderConfig::ISSUER_URL)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::ISSUER_URL, error.to_string()))?;
        let issuer_url = Url::parse(&issuer)
            .map_err(|error| anyhow!("Failed to parse issuer URL: {}", error.to_string()))?;
        let raw_scopes = config.get_string(OidcIdentityProviderConfig::SCOPES)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::SCOPES, error.to_string()))?;
        let scopes = OidcIdentityProviderConfig::parse_scopes(&client_id, raw_scopes);

        Ok(Self {
            client_id: OAuthClientId::new(client_id),
            client_secret: OAuthClientSecret::new(client_secret),
            issuer_url,
            scopes,
        })
    }
}
