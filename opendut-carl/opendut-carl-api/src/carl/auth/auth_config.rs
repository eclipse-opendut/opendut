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
        let raw_scopes_no_quotations = raw_scopes.replace('\"', "");
        let scopes = raw_scopes_no_quotations.split(',').collect::<Vec<_>>();
        for scope in scopes.clone() {
            if !scope.chars().all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit()) {
                panic!("Failed to parse comma-separated OIDC scopes for client_id='{}'. Scopes must only contain ASCII alphabetic characters or digits. Found: {:?}. Parsed as: {:?}", client_id, raw_scopes, scopes);
            }
        }
        scopes.into_iter().filter(|scope| !scope.is_empty()).map(|scope| OAuthScope::new(scope.to_string())).collect()

    }

}

impl TryFrom<&Config> for OidcIdentityProviderConfig {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> anyhow::Result<Self> {
        let client_id = config.get_string(OidcIdentityProviderConfig::CLIENT_ID)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::CLIENT_ID, error))?;
        let client_secret = config.get_string(OidcIdentityProviderConfig::CLIENT_SECRET)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::CLIENT_SECRET, error))?;
        let issuer = config.get_string(OidcIdentityProviderConfig::ISSUER_URL)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::ISSUER_URL, error))?;
        let issuer_url = Url::parse(&issuer)
            .map_err(|error| anyhow!("Failed to parse issuer URL: {}", error))?;
        if issuer_url.as_str().ends_with('/') {
            let raw_scopes = config.get_string(OidcIdentityProviderConfig::SCOPES)
                .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", OidcIdentityProviderConfig::SCOPES, error))?;
            let scopes = OidcIdentityProviderConfig::parse_scopes(&client_id, raw_scopes);

            Ok(Self {
                client_id: OAuthClientId::new(client_id),
                client_secret: OAuthClientSecret::new(client_secret),
                issuer_url,
                scopes,
            })
        } else {
            Err(anyhow!("Issuer URL must end with a `/`. Found: {}", issuer_url))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::carl::auth::auth_config::OidcIdentityProviderConfig;

    #[test]
    fn test_parse_scopes() {
        let client_id = "test_client_id";
        let raw_scopes = "scope1,scope2,scope3";
        let scopes = OidcIdentityProviderConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].as_str(), "scope1");
        assert_eq!(scopes[1].as_str(), "scope2");
        assert_eq!(scopes[2].as_str(), "scope3");
    }

    #[test]
    fn test_parse_empty_scopes() {
        let client_id = "test_client_id";
        let raw_scopes = ",";
        let scopes = OidcIdentityProviderConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 0);
    }

    #[test]
    fn test_parse_empty_quoted_scopes() {
        let client_id = "test_client_id";
        let raw_scopes = "\"\"";
        let scopes = OidcIdentityProviderConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_scope_delimiter() {
        let client_id = "test_client_id";
        let raw_scopes = "foo asd";
        let _ = OidcIdentityProviderConfig::parse_scopes(client_id, raw_scopes.to_string());
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_scope_characters() {
        let client_id = "test_client_id";
        let raw_scopes = "foo!bar";
        let _ = OidcIdentityProviderConfig::parse_scopes(client_id, raw_scopes.to_string());
    }

    #[test]
    fn test_parse_scopes_should_ignore_quotations() {
        let client_id = "test_client_id";
        let raw_scopes = "\"scope1\",\"scope2\",\"scope3\"";
        let scopes = OidcIdentityProviderConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].as_str(), "scope1");
        assert_eq!(scopes[1].as_str(), "scope2");
        assert_eq!(scopes[2].as_str(), "scope3");
    }


}
