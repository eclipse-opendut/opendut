use std::fmt::Formatter;
use std::sync::Arc;
use std::time::Duration;
use chrono::{NaiveDateTime, Utc};
use config::Config;
use oauth2::{AccessToken, TokenResponse};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use std::sync::{RwLock, RwLockWriteGuard};
use tokio::task::JoinError;
use tracing::debug;
use crate::confidential::config::{ConfidentialClientConfig, ConfidentialClientConfigData};
use crate::confidential::blocking_reqwest_client::OidcBlockingReqwestClient;
use crate::confidential::error::{ConfidentialClientError, WrappedRequestTokenError};

#[derive(Debug)]
pub struct ConfidentialClient {
    inner: BasicClient,
    pub reqwest_client: OidcBlockingReqwestClient,
    pub config: ConfidentialClientConfigData,

    state: RwLock<Option<TokenStorage>>,
}

#[derive(Debug, Clone)]
struct TokenStorage {
    pub access_token: AccessToken,
    pub expires_in: NaiveDateTime,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("FailedToGetToken: {message} cause: {cause}.")]
    FailedToGetToken { message: String, cause: WrappedRequestTokenError },
    #[error("ExpirationFieldMissing: {message}.")]
    ExpirationFieldMissing { message: String },
    #[error("FailedToUpdateToken: {message} cause: {cause}.")]
    FailedToUpdateToken { message: String, cause: JoinError },
    
}

#[derive(Clone)]
pub struct Token {
    pub value: String,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.value)
    }
}

impl Token {
    pub fn new(value: String) -> Self {
        Token { value }
    }
    pub fn oauth_token(&self) -> AccessToken {
        AccessToken::new(self.value.clone())
    }
}

pub type ConfidentialClientRef = Arc<ConfidentialClient>;

impl ConfidentialClient {
    pub async fn from_settings(settings: &Config) -> Result<Option<ConfidentialClientRef>, ConfidentialClientError> {
        let client_config = ConfidentialClientConfig::from_settings(settings)
            .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to load OIDC configuration"), cause: cause.into() })?;

        match client_config {
            ConfidentialClientConfig::Confidential(client_config) => {
                debug!("OIDC configuration loaded: id={:?} issuer_url={:?}", client_config.client_id, client_config.issuer_url);

                debug!("Connecting to KEYCLOAK...");

                let connection_result = ConfidentialClient::check_connection(client_config.clone()).await;
                
                match connection_result {
                    Ok(_) => {
                        let reqwest_client = OidcBlockingReqwestClient::from_config(settings).await
                            .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to create reqwest client."), cause: cause.into() })?;

                        let client = ConfidentialClient::from_client_config(client_config, reqwest_client).await?;

                        Ok(Some(client))
                    }
                    Err(error) => {
                        Err(error)
                    }
                }
            }
            ConfidentialClientConfig::AuthenticationDisabled => {
                debug!("OIDC is disabled.");
                Ok(None)
            }
        }
    }

    pub async fn from_client_config(client_config: ConfidentialClientConfigData, reqwest_client: OidcBlockingReqwestClient) -> Result<ConfidentialClientRef, ConfidentialClientError> {
        let inner = client_config.get_client()?;

        let client = Self {
            inner,
            reqwest_client,
            config: client_config,
            state: Default::default(),
        };
        Ok(Arc::new(client))
    }

    async fn check_connection(idp_config: ConfidentialClientConfigData) -> Result<(), ConfidentialClientError> {

        let token_endpoint = idp_config.issuer_url.join("protocol/openid-connect/token")
            .map_err(|error| ConfidentialClientError::UrlParse { message: String::from("Failed to derive token url from issuer url: "), cause: error })?;

        let token_endpoint_copy = token_endpoint.clone();

        let mut error: Option<reqwest::Error> = None;

        const MAX_RETRIES: u16 = 5;
        for _retries_left in (0..MAX_RETRIES).rev() {
            let token_url = token_endpoint_copy.clone();
            let response = reqwest::get(token_url.clone()).await;
            match response {
                Ok(_) => {
                    return Ok(());
                }
                Err(cause) => {
                    error = Some(cause);
                    tokio::time::sleep(Duration::from_millis(10000)).await;
                    continue;
                }
            };
        }
        Err(ConfidentialClientError::KeycloakConnection { message: String::from("Could not connect to keycloak"), cause: error.unwrap() })
    }

    fn update_storage_token(response: &BasicTokenResponse, state: &mut RwLockWriteGuard<Option<TokenStorage>>) -> Result<Token, AuthError> {
        let access_token = response.access_token().clone();
        let expires_in = match response.expires_in() {
            None => {
                return Err(AuthError::ExpirationFieldMissing { message: "No expires_in in response.".to_string() });
            }
            Some(expiry_duration) => { Utc::now().naive_utc() + expiry_duration }
        };
        let _token_storage = state.insert(TokenStorage {
            access_token,
            expires_in,
        });
        Ok(Token { value: response.access_token().secret().to_string() })
    }

    fn fetch_token(&self) -> Result<Token, AuthError> {
        let response = self.inner.exchange_client_credentials()
            .add_scopes(self.config.scopes.clone())
            .request(|request| { self.reqwest_client.sync_http_client(request) })
            .map_err(|error| {
                AuthError::FailedToGetToken {
                    message: "Fetching authentication token failed!".to_string(),
                    cause: WrappedRequestTokenError(error),
                }
            })?;

        let mut state = self.state.write().expect("Failed to rewrite the token."); //TODO
        
        Self::update_storage_token(&response, &mut state)?;
        Ok(Token { value: response.access_token().secret().to_string() })
    }

    pub fn get_token(&self) -> Result<Token, AuthError> {
        let token_storage = self.state.read().unwrap().clone();
        let access_token = match token_storage {
            None => {
                self.fetch_token()?
            }
            Some(token) => {
                if Utc::now().naive_utc().lt(&token.expires_in) {
                    Token { value: token.access_token.secret().to_string() }
                } else {
                    self.fetch_token()?
                }
            }
        };
        Ok(access_token)
    }
}

#[cfg(test)]
mod auth_tests {
    use chrono::Utc;
    use googletest::assert_that;
    use googletest::matchers::gt;
    use oauth2::{ClientId, ClientSecret, TokenResponse};
    use url::Url;
    use opendut_util_core::project;
    use crate::confidential::config::ConfidentialClientConfigData;
    use crate::confidential::pem::read_pem_from_file_path;
    use crate::confidential::blocking_reqwest_client;

    #[test]
    fn test_get_token_example() {
        let client_config = ConfidentialClientConfigData::new(
            ClientId::new("opendut-edgar-client".to_string()),
            ClientSecret::new("c7d6ace0-b90f-471a-bb62-a4ecac4150f8".to_string()),
            Url::parse("http://localhost:8081/realms/opendut/").unwrap(),
            vec![],
        );
        let ca_path = project::make_path_absolute("resources/development/tls/insecure-development-ca.pem")
            .expect("Could not resolve dev CA");
        let client = client_config.get_client().unwrap();
        
        let certificate = read_pem_from_file_path(&ca_path).unwrap();
        let reqwest_client = blocking_reqwest_client::OidcBlockingReqwestClient::from_pem(certificate).unwrap();
        let response = client.exchange_client_credentials()
            .add_scopes(vec![])
            .request(|request| reqwest_client.sync_http_client(request))
            .expect("Failed to get token");
        
        let now = Utc::now().naive_utc();
        let expires_in = now + response.expires_in().expect("Expiration field missing!");
        let access_token = response.access_token();
        let token = access_token
            .secret()
            .to_string();
        assert_that!(token.len(), gt(10));
        assert!(now.lt(&expires_in));
    }
}
