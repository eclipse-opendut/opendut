use std::fmt::Formatter;
use std::sync::Arc;
use chrono::{NaiveDateTime, Utc};
use config::Config;
use oauth2::{AccessToken, TokenResponse};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use tokio::sync::{RwLock, RwLockWriteGuard};
use tracing::debug;
use crate::confidential::config::{ConfidentialClientConfig, ConfidentialClientConfigData};
use crate::confidential::reqwest_client::OidcReqwestClient;
use crate::confidential::error::{ConfidentialClientError, WrappedRequestTokenError};

#[derive(Debug)]
pub struct ConfidentialClient {
    inner: BasicClient,
    pub reqwest_client: OidcReqwestClient,
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
                let reqwest_client = OidcReqwestClient::from_config(settings).await
                    .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to create reqwest client."), cause: cause.into() })?;
                let client = ConfidentialClient::from_client_config(client_config, reqwest_client).await?;
                Ok(Some(client))
            }
            ConfidentialClientConfig::AuthenticationDisabled => {
                debug!("OIDC is disabled.");
                Ok(None)
            }
        }
    }

    pub async fn from_client_config(client_config: ConfidentialClientConfigData, reqwest_client: OidcReqwestClient) -> Result<ConfidentialClientRef, ConfidentialClientError> {
        let inner = client_config.get_client()?;

        let client = Self {
            inner,
            reqwest_client,
            config: client_config,
            state: Default::default(),
        };
        Ok(Arc::new(client))
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

    async fn fetch_token(&self) -> Result<Token, AuthError> {
        let response = self.inner.exchange_client_credentials()
            .add_scopes(self.config.scopes.clone())
            .request_async(|request| { self.reqwest_client.async_http_client(request) })
            .await
            .map_err(|error| {
                AuthError::FailedToGetToken {
                    message: "Fetching authentication token failed!".to_string(),
                    cause: WrappedRequestTokenError(error),
                }
            })?;

        let mut state = self.state.write().await;

        Self::update_storage_token(&response, &mut state)?;

        Ok(Token { value: response.access_token().secret().to_string() })
    }

    pub async fn get_token(&self) -> Result<Token, AuthError> {
        let token_storage = self.state.read().await.clone();
        let access_token = match token_storage {
            None => {
                self.fetch_token().await?
            }
            Some(token) => {
                if Utc::now().naive_utc().lt(&token.expires_in) {
                    Token { value: token.access_token.secret().to_string() }
                } else {
                    self.fetch_token().await?
                }
            }
        };
        Ok(access_token)
    }

    pub async fn check_login(&self) -> Result<bool, AuthError> {
        let token = self.get_token().await?;
        Ok(!token.value.is_empty())
    }
}

#[cfg(test)]
mod auth_tests {
    use anyhow::anyhow;
    use oauth2::{ClientId, ClientSecret};
    use pem::Pem;
    use rstest::{fixture, rstest};
    use url::Url;
    use opendut_util_core::project;
    use crate::confidential::config::ConfidentialClientConfigData;
    use crate::confidential::client::{ConfidentialClient, ConfidentialClientRef};
    use crate::confidential::pem::PemFromConfig;
    use crate::confidential::reqwest_client::OidcReqwestClient;

    #[fixture]
    async fn confidential_edgar_client() -> ConfidentialClientRef {
        let client_config = ConfidentialClientConfigData::new(
            ClientId::new("opendut-edgar-client".to_string()),
            ClientSecret::new("c7d6ace0-b90f-471a-bb62-a4ecac4150f8".to_string()),
            Url::parse("http://localhost:8081/realms/opendut/").unwrap(),
            vec![],
        );
        let ca_path = project::make_path_absolute("resources/development/tls/insecure-development-ca.pem")
            .expect("Could not resolve dev CA").into_os_string().into_string().unwrap();
        let result = <Pem as PemFromConfig>::from_file_path(&ca_path).await;
        let pem: Pem = result.expect("Could not load dev CA");
        let reqwest_client = OidcReqwestClient::from_pem(pem)
            .map_err(|cause| anyhow!("Failed to create reqwest client. Error: {}", cause)).unwrap();

        ConfidentialClient::from_client_config(client_config, reqwest_client).await.unwrap()
    }

    #[rstest]
    #[tokio::test]
    #[ignore]
    async fn test_confidential_client_get_token(#[future] confidential_edgar_client: ConfidentialClientRef) {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command:
         * cargo test --package opendut-auth --all-features -- --include-ignored
         */
        let token = confidential_edgar_client.await.get_token().await.unwrap();
        assert!(token.value.len() > 100);
    }
}
