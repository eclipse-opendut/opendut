use anyhow::anyhow;
use config::Config;
use oauth2::{AccessToken, AuthUrl, ClientId as OAuthClientId, ClientSecret as OAuthClientSecret, Scope as OAuthScope, RedirectUrl, TokenResponse, TokenUrl};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use openidconnect::core::{CoreClientRegistrationRequest, CoreGrantType};
use openidconnect::registration::EmptyAdditionalClientMetadata;
use openidconnect::RegistrationUrl;
use serde::{Deserialize, Serialize};
use shadow_rs::formatcp;
use tracing::debug;
use url::Url;

use opendut_carl_api::carl::auth::auth_config::OidcIdentityProviderConfig;
use opendut_types::util::net::{ClientCredentials, ClientId, ClientSecret};

pub const DEVICE_REDIRECT_URL: &str = "http://localhost:12345/device";

#[derive(Debug, Clone)]
pub struct OpenIdConnectClientManager {
    client: BasicClient,
    registration_url: RegistrationUrl,
    device_redirect_url: RedirectUrl,
    pub issuer_url: Url,
    pub issuer_remote_url: Url,
    peer_credentials: Option<CommonPeerCredentials>
}

#[derive(Debug)]
pub struct OAuthClientCredentials {
    pub client_id: OAuthClientId,
    pub client_secret: OAuthClientSecret,
}

impl From<OAuthClientCredentials> for ClientCredentials {
    fn from(value: OAuthClientCredentials) -> Self {
        Self {
            client_id: ClientId(value.client_id.to_string()),
            client_secret: ClientSecret(value.client_secret.secret().to_string()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CarlScopes(pub String);

const CARL_OIDC_CONFIG_PREFIX: &str = "network.oidc.client";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonPeerCredentials {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CarlIdentityProviderConfig {
    client_id: OAuthClientId,
    client_secret: OAuthClientSecret,
    issuer_url: Url,
    issuer_remote_url: Url,
    scopes: Vec<OAuthScope>,
    peer_credentials: Option<CommonPeerCredentials>
}

impl TryFrom<&Config> for CarlIdentityProviderConfig {
    type Error = anyhow::Error;

    fn try_from(config: &Config) -> anyhow::Result<Self> {
        let client_id = config.get_string(CarlIdentityProviderConfig::CLIENT_ID)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", CarlIdentityProviderConfig::CLIENT_ID, error))?;
        let client_secret = config.get_string(CarlIdentityProviderConfig::CLIENT_SECRET)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", CarlIdentityProviderConfig::CLIENT_SECRET, error))?;
        let issuer = config.get_string(CarlIdentityProviderConfig::ISSUER_URL)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", CarlIdentityProviderConfig::ISSUER_URL, error))?;
        let issuer_remote = config.get_string(CarlIdentityProviderConfig::ISSUER_REMOTE_URL)
            .map_err(|error| anyhow!("Failed to find configuration for `{}`. {}", CarlIdentityProviderConfig::ISSUER_REMOTE_URL, error))?;

        let peer_id = config.get_string(CarlIdentityProviderConfig::COMMON_PEER_ID).ok();
        let peer_secret = config.get_string(CarlIdentityProviderConfig::COMMON_PEER_SECRET).ok();

        let peer_credentials = match (peer_id, peer_secret) {
            (Some(id), Some(secret)) => {
                log::debug!("Using defined common peer credentials for all peers with id='{}'", id);
                Some(CommonPeerCredentials {
                    client_id: ClientId(id),
                    client_secret: ClientSecret(secret),
                })
            }
            _ => None
        };

        let issuer_url = Url::parse(&issuer)
            .map_err(|error| anyhow!("Failed to parse issuer URL: {}", error))?;
        let issuer_remote_url = Url::parse(&issuer_remote)
            .map_err(|error| anyhow!("Failed to parse issuer remote URL: {}", error))?;

        let raw_scopes = config.get_string(CarlIdentityProviderConfig::SCOPES).unwrap_or_default();

        Ok(Self {
            client_id: OAuthClientId::new(client_id.clone()),
            client_secret: OAuthClientSecret::new(client_secret),
            issuer_url,
            issuer_remote_url,
            scopes: OidcIdentityProviderConfig::parse_scopes(&client_id, raw_scopes),
            peer_credentials,
        })
    }
}

impl CarlIdentityProviderConfig {
    const CLIENT_ID: &'static str = formatcp!("{CARL_OIDC_CONFIG_PREFIX}.client.id");
    const CLIENT_SECRET: &'static str = formatcp!("{CARL_OIDC_CONFIG_PREFIX}.client.secret");
    const COMMON_PEER_ID: &'static str = formatcp!("{CARL_OIDC_CONFIG_PREFIX}.peer.id");
    const COMMON_PEER_SECRET: &'static str = formatcp!("{CARL_OIDC_CONFIG_PREFIX}.peer.secret");
    const ISSUER_URL: &'static str = formatcp!("{CARL_OIDC_CONFIG_PREFIX}.issuer.url");
    const ISSUER_REMOTE_URL: &'static str = formatcp!("{CARL_OIDC_CONFIG_PREFIX}.issuer.remote.url");
    const SCOPES: &'static str = formatcp!("{CARL_OIDC_CONFIG_PREFIX}.scopes");
}

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationClientManagerError {
    #[error("Invalid configuration:\n  {error}")]
    InvalidConfiguration {
        error: String,
    },
    #[error("Invalid client credentials:\n  {error}")]
    InvalidCredentials {
        error: String,
    },
    #[error("Failed to register new client:\n  {error}")]
    Registration {
        error: String,
    },
}

impl OpenIdConnectClientManager {
    /// issuer_url for keycloak includes realm name: http://localhost:8081/realms/opendut
    pub fn new(config: CarlIdentityProviderConfig) -> Result<Self, AuthenticationClientManagerError> {
        if config.issuer_url.as_str().ends_with('/') {
            // keycloak auth url: http://localhost:8081/realms/opendut/protocol/openid-connect/auth
            let auth_url = AuthUrl::from_url(
                config.issuer_url.join("protocol/openid-connect/auth")
                    .map_err(|error| AuthenticationClientManagerError::InvalidConfiguration { error: format!("Invalid auth endpoint url: {}", error) })?
            );
            let token_url = TokenUrl::from_url(
                config.issuer_url.join("protocol/openid-connect/token")
                    .map_err(|error| AuthenticationClientManagerError::InvalidConfiguration { error: format!("Invalid token endpoint url: {}", error) })?
            );
            let registration_url = RegistrationUrl::from_url(
                config.issuer_url.join("clients-registrations/openid-connect")
                    .map_err(|error| AuthenticationClientManagerError::InvalidConfiguration { error: format!("Invalid registration endpoint URL: {}", error) })?
            );

            let device_redirect_url = RedirectUrl::new(DEVICE_REDIRECT_URL.to_string()).expect("Could not parse device redirect url");

            let client =
                BasicClient::new(
                    config.client_id,
                    Some(config.client_secret),
                    auth_url,
                    Some(token_url),
                );

            let manager = Ok(OpenIdConnectClientManager {
                client,
                registration_url,
                device_redirect_url,
                issuer_url: config.issuer_url.clone(),
                issuer_remote_url: config.issuer_remote_url.clone(),
                peer_credentials: config.peer_credentials,
            });
            debug!("Created OpenIdConnectClientManager: {:?}", manager);
            manager
        } else {
            Err(AuthenticationClientManagerError::InvalidConfiguration {
                error: "Issuer URL must end with a slash".to_string(),
            })
        }
    }

    async fn get_token(&self) -> Result<AccessToken, AuthenticationClientManagerError> {
        let response = self.client.exchange_client_credentials()
            .request_async(async_http_client)
            .await
            .map_err(|error| AuthenticationClientManagerError::InvalidCredentials { error: error.to_string() })?;
        Ok(response.access_token().clone())
    }

    pub async fn register_new_client(&self) -> Result<OAuthClientCredentials, AuthenticationClientManagerError> {
        match self.peer_credentials.clone() {
            Some(peer_credentials) => {
                Ok(OAuthClientCredentials {
                    client_id: OAuthClientId::new(peer_credentials.client_id.value()),
                    client_secret: OAuthClientSecret::new(peer_credentials.client_secret.value()),
                })
            }
            None => {
                let access_token = self.get_token().await?;
                let additional_metadata = EmptyAdditionalClientMetadata {};
                let redirect_uris = vec![self.device_redirect_url.clone()];
                let grant_types = vec![CoreGrantType::ClientCredentials];
                let request: CoreClientRegistrationRequest =
                    openidconnect::registration::ClientRegistrationRequest::new(redirect_uris, additional_metadata)
                        .set_grant_types(Some(grant_types));
                let registration_url = self.registration_url.clone();
                let response = request
                    .set_initial_access_token(Some(access_token))
                    .register_async(&registration_url, async_http_client).await;

                match response {
                    Ok(response) => {
                        let client_id = response.client_id();
                        let client_secret = response.client_secret().expect("Confidential client required!");

                        Ok(OAuthClientCredentials {
                            client_id: client_id.clone(),
                            client_secret: client_secret.clone(),
                        })
                    }
                    Err(error) => {
                        Err(AuthenticationClientManagerError::Registration {
                            error: format!("{:?}", error),
                        })
                    }
                }
            }
        }
    }
}


#[cfg(test)]
pub mod tests {
    use googletest::assert_that;
    use googletest::matchers::eq;
    use http::{HeaderMap, HeaderValue};
    use oauth2::HttpRequest;
    use rstest::{fixture, rstest};
    use url::Url;

    use super::*;

    async fn delete_client(manager: OpenIdConnectClientManager, client_id: &OAuthClientId) -> Result<(), AuthenticationClientManagerError> {
        let access_token = manager.get_token().await?;
        let request_base_url: Url = "http://localhost:8081/admin/realms/opendut/clients/".parse().unwrap();
        let delete_client_url = request_base_url.join(&format!("{}", client_id.to_string()))
            .map_err(|error| AuthenticationClientManagerError::InvalidConfiguration { error: format!("Invalid client URL: {}", error) })?;

        let mut headers = HeaderMap::new();
        let bearer_header = format!("Bearer {}", access_token.secret().as_str());
        let access_token_value = HeaderValue::from_str(&bearer_header)
            .map_err(|error| AuthenticationClientManagerError::InvalidConfiguration { error: error.to_string() })?;
        headers.insert(http::header::AUTHORIZATION, access_token_value);

        let request = HttpRequest {
            method: http::Method::DELETE,
            url: delete_client_url,
            headers,
            body: vec![],
        };
        let response = async_http_client(request)
            .await
            .map_err(|error| AuthenticationClientManagerError::Registration { error: error.to_string() })?;
        assert_eq!(response.status_code, 204, "Failed to delete client with id '{:?}': {:?}", client_id, response.body);

        Ok(())
    }

    #[fixture]
    pub fn oidc_client_manager() -> OpenIdConnectClientManager {
        /*
         * Issuer URL for keycloak needs to align with FRONTEND_URL in Keycloak realm setting.
         * Localhost address is always fine, though.
         */
        let client_id = "opendut-carl-client".to_string();
        let client_secret = "6754d533-9442-4ee6-952a-97e332eca38e".to_string();
        //let issuer_url = "http://192.168.56.10:8081/realms/opendut/".to_string();  // This is the URL for the keycloak server in the test environment (valid in host system and opendut-vm)
        let issuer_url = "https://keycloak/realms/opendut/".to_string();  // This is the URL for the keycloak server in the test environment
//         let issuer_url = "http://localhost:8081/realms/opendut/".to_string();
        let issuer_remote_url = "https://keycloak/realms/opendut/".to_string();  // works inside OpenDuT-VM
        let carl_idp_config = CarlIdentityProviderConfig {
            client_id: OAuthClientId::new(client_id),
            client_secret: OAuthClientSecret::new(client_secret),
            issuer_url: Url::parse(&issuer_url).unwrap(),
            issuer_remote_url: Url::parse(&issuer_remote_url).unwrap(),
            scopes: vec![],
            peer_credentials: None,
        };
        OpenIdConnectClientManager::new(carl_idp_config).unwrap()
    }

    #[rstest]
    #[tokio::test]
    #[ignore]
    async fn test_register_new_oidc_client(oidc_client_manager: OpenIdConnectClientManager) {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command: cargo test -- --include-ignored
         */
        println!("{:?}", oidc_client_manager);
        let credentials = oidc_client_manager.register_new_client().await.unwrap();
        println!("New client id: {}, secret: {}", credentials.client_id.to_string(), credentials.client_secret.secret().to_string());
        delete_client(oidc_client_manager, &credentials.client_id).await.unwrap();
        assert_that!(credentials.client_id.to_string().len().gt(&10), eq(true));

    }
}
