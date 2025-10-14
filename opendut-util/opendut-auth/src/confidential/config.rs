use config::Config;
use oauth2::{AuthUrl, EndpointNotSet, EndpointSet, TokenUrl};
use oauth2::{ClientId as OAuthClientId, ClientSecret as OAuthClientSecret, Scope as OAuthScope};
use oauth2::{ResourceOwnerPassword as OAuthResourceOwnerPassword, ResourceOwnerUsername as OAuthResourceOwnerUsername};
use oauth2::basic::BasicClient;
use const_format::formatcp;
use url::Url;

use crate::confidential::error::ConfidentialClientError;

pub type ConfiguredClient<
    HasAuthUrl = EndpointSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointSet,
> = BasicClient<
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
>;

#[derive(Clone, Debug)]
pub enum OidcClientConfig {
    Confidential(OidcConfidentialClientConfig),
    ResourceOwner(OidcResourceOwnerConfidentialClientConfig),
    AuthenticationDisabled,
}

impl OidcClientConfig {
    pub fn from_settings(settings: &Config) -> Result<Self, ConfidentialClientError> {
        let oidc_enabled = settings.get_bool(CONFIG_KEY_OIDC_ENABLED)
            .map_err(|cause| ConfidentialClientError::Configuration { message: format!("No configuration found for {CONFIG_KEY_OIDC_ENABLED}."), cause: cause.into() })?;
        if oidc_enabled {
            Ok(Self::Confidential(OidcConfidentialClientConfig::from_settings(settings)?))
        } else {
            Ok(Self::AuthenticationDisabled)
        }
    }
}

#[derive(Clone, Debug)]
pub struct OidcConfidentialClientConfig {
    pub client_id: OAuthClientId,
    client_secret: OAuthClientSecret,
    pub issuer_url: Url,
    pub scopes: Vec<OAuthScope>,
}

#[derive(Clone, Debug)]
pub struct OidcResourceOwnerConfidentialClientConfig {
    pub client_id: OAuthClientId,
    client_secret: OAuthClientSecret,
    pub issuer_url: Url,
    pub scopes: Vec<OAuthScope>,
    pub(crate) username: OAuthResourceOwnerUsername,
    pub(crate) password: OAuthResourceOwnerPassword,
}

impl OidcResourceOwnerConfidentialClientConfig {
    pub fn new(client_id: OAuthClientId, client_secret: OAuthClientSecret, issuer_url: Url, scopes: Vec<OAuthScope>, username: String, password: String) -> Self {
        Self {
            client_id,
            client_secret,
            issuer_url,
            scopes,
            username: OAuthResourceOwnerUsername::new(username),
            password: OAuthResourceOwnerPassword::new(password),
        }
    }
    
    // TODO: fix duplicate
    pub fn get_client(&self) -> Result<ConfiguredClient, ConfidentialClientError> {
        let auth_endpoint = self.issuer_url.join("protocol/openid-connect/auth")
            .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to derive authorization url from issuer url."), cause: cause.into() })?;
        let token_endpoint = self.issuer_url.join("protocol/openid-connect/token")
            .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to derive token url from issuer url."), cause: cause.into() })?;

        let client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(AuthUrl::from_url(auth_endpoint))
            .set_token_uri(TokenUrl::from_url(token_endpoint));

        Ok(client)
    }
}


pub const CONFIG_KEY_OIDC_ENABLED: &str = "network.oidc.enabled";
const OIDC_CLIENT_CONFIG_PREFIX: &str = "network.oidc.client";

impl OidcConfidentialClientConfig {
    const CLIENT_ID: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.id");
    const CLIENT_SECRET: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.secret");
    const ISSUER_URL: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.issuer.url");
    const SCOPES: &'static str = formatcp!("{OIDC_CLIENT_CONFIG_PREFIX}.scopes");

    pub fn parse_scopes(client_id: &str, raw_scopes: String) -> Vec<OAuthScope> {
        let raw_scopes_no_quotations = raw_scopes.replace('\"', "");
        let scopes = raw_scopes_no_quotations.split(',').collect::<Vec<_>>();
        for scope in scopes.clone() {
            if !scope.chars().all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit()) {
                panic!("Failed to parse comma-separated OIDC scopes for client_id='{client_id}'. Scopes must only contain ASCII alphabetic characters or digits. Found: {raw_scopes:?}. Parsed as: {scopes:?}");
            }
        }
        scopes.into_iter().filter(|scope| !scope.is_empty()).map(|scope| OAuthScope::new(scope.to_string())).collect()
    }
    pub fn new(client_id: OAuthClientId, client_secret: OAuthClientSecret, issuer_url: Url, scopes: Vec<OAuthScope>) -> Self {
        Self {
            client_id,
            client_secret,
            issuer_url,
            scopes,
        }
    }

    pub fn get_client(&self) -> Result<ConfiguredClient, ConfidentialClientError> {
        let auth_endpoint = self.issuer_url.join("protocol/openid-connect/auth")
            .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to derive authorization url from issuer url."), cause: cause.into() })?;
        let token_endpoint = self.issuer_url.join("protocol/openid-connect/token")
            .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to derive token url from issuer url."), cause: cause.into() })?;

        let client = BasicClient::new(self.client_id.clone())
            .set_client_secret(self.client_secret.clone())
            .set_auth_uri(AuthUrl::from_url(auth_endpoint))
            .set_token_uri(TokenUrl::from_url(token_endpoint));

        Ok(client)
    }
}

impl OidcConfidentialClientConfig {
    pub fn from_settings(settings: &Config) -> Result<Self, ConfidentialClientError> {
        let client_id = settings.get_string(OidcConfidentialClientConfig::CLIENT_ID)
            .map_err(|error| ConfidentialClientError::Configuration { message: format!("Failed to find configuration for `{}`.", OidcConfidentialClientConfig::CLIENT_ID), cause: error.into() })?;
        let client_secret = settings.get_string(OidcConfidentialClientConfig::CLIENT_SECRET)
            .map_err(|error| ConfidentialClientError::Configuration { message: format!("Failed to find configuration for `{}`.", OidcConfidentialClientConfig::CLIENT_SECRET), cause: error.into() })?;
        let issuer = settings.get_string(OidcConfidentialClientConfig::ISSUER_URL)
            .map_err(|error| ConfidentialClientError::Configuration { message: format!("Failed to find configuration for `{}`.", OidcConfidentialClientConfig::ISSUER_URL), cause: error.into() })?;

        let issuer_url = Url::parse(&issuer)
            .map_err(|error| ConfidentialClientError::Configuration { message: format!("Failed to parse issuer URL: `{issuer}`."), cause: error.into() })?;
        // TODO: add validation for issuer url to new type
        if issuer_url.as_str().ends_with('/') {
            let raw_scopes = settings.get_string(OidcConfidentialClientConfig::SCOPES)
                .map_err(|error| ConfidentialClientError::Configuration { message: format!("Failed to find configuration for `{}`.", OidcConfidentialClientConfig::SCOPES), cause: error.into() })?;
            let scopes = OidcConfidentialClientConfig::parse_scopes(&client_id, raw_scopes);

            Ok(Self {
                client_id: OAuthClientId::new(client_id),
                client_secret: OAuthClientSecret::new(client_secret),
                issuer_url,
                scopes,
            })
        } else {
            Err(ConfidentialClientError::Other { message: format!("Issuer URL must end with a `/`. Found: {issuer_url}") })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::confidential::config::OidcConfidentialClientConfig;

    #[test]
    fn test_parse_scopes() {
        let client_id = "test_client_id";
        let raw_scopes = "scope1,scope2,scope3";
        let scopes = OidcConfidentialClientConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].as_str(), "scope1");
        assert_eq!(scopes[1].as_str(), "scope2");
        assert_eq!(scopes[2].as_str(), "scope3");
    }

    #[test]
    fn test_parse_empty_scopes() {
        let client_id = "test_client_id";
        let raw_scopes = ",";
        let scopes = OidcConfidentialClientConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 0);
    }

    #[test]
    fn test_parse_empty_quoted_scopes() {
        let client_id = "test_client_id";
        let raw_scopes = "\"\"";
        let scopes = OidcConfidentialClientConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 0);
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_scope_delimiter() {
        let client_id = "test_client_id";
        let raw_scopes = "foo asd";
        let _ = OidcConfidentialClientConfig::parse_scopes(client_id, raw_scopes.to_string());
    }

    #[test]
    #[should_panic]
    fn test_parse_invalid_scope_characters() {
        let client_id = "test_client_id";
        let raw_scopes = "foo!bar";
        let _ = OidcConfidentialClientConfig::parse_scopes(client_id, raw_scopes.to_string());
    }

    #[test]
    fn test_parse_scopes_should_ignore_quotations() {
        let client_id = "test_client_id";
        let raw_scopes = "\"scope1\",\"scope2\",\"scope3\"";
        let scopes = OidcConfidentialClientConfig::parse_scopes(client_id, raw_scopes.to_string());
        assert_eq!(scopes.len(), 3);
        assert_eq!(scopes[0].as_str(), "scope1");
        assert_eq!(scopes[1].as_str(), "scope2");
        assert_eq!(scopes[2].as_str(), "scope3");
    }
}
