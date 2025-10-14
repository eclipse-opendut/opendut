use async_trait::async_trait;
use crate::confidential::client::{AuthError, ConfidentialClient, SharedTokenStorage, Token};
use crate::confidential::config::{ConfiguredClient, OidcConfidentialClientConfig, OidcResourceOwnerConfidentialClientConfig};
use crate::confidential::error::WrappedRequestTokenError;
use crate::confidential::reqwest_client::async_http_client;
use oauth2::Scope as OAuthScope;
use oauth2::TokenResponse;
use opendut_util_core::future::ExplicitSendFutureWrapper;


#[async_trait::async_trait]
pub trait OidcAuthenticator: Send + Sync {
    async fn fetch_token(&self, client: &ConfiguredClient, scopes: Vec<OAuthScope>,
                         reqwest_client: &reqwest::Client, token_store: SharedTokenStorage) -> Result<Token, AuthError>;
}


#[async_trait]
impl OidcAuthenticator for OidcConfidentialClientConfig {
    async fn fetch_token(&self, client: &ConfiguredClient, scopes: Vec<OAuthScope>,
                         reqwest_client: &reqwest::Client, token_store: SharedTokenStorage) -> Result<Token, AuthError> {
        let response = ExplicitSendFutureWrapper::from(
            client.exchange_client_credentials()
                .add_scopes(scopes)
                .request_async(&|request| { async_http_client(reqwest_client, request) })
        ).await
            .map_err(|error| {
                AuthError::FailedToGetToken {
                    message: "Fetching authentication token failed!".to_string(),
                    cause: WrappedRequestTokenError(error),
                }
            })?;

        let mut state = token_store.write().await;
        ConfidentialClient::update_storage_token(&response, &mut state)?;

        Ok(Token { value: response.access_token().secret().to_string() })
    }
}

#[async_trait]
impl OidcAuthenticator for OidcResourceOwnerConfidentialClientConfig {
    async fn fetch_token(&self, client: &ConfiguredClient, scopes: Vec<OAuthScope>,
                         reqwest_client: &reqwest::Client, token_store: SharedTokenStorage) -> Result<Token, AuthError> {
        let response = ExplicitSendFutureWrapper::from(
            client.exchange_password(&self.username, &self.password)
                .add_scopes(scopes)
                .request_async(&|request| { async_http_client(reqwest_client, request) })
        ).await
            .map_err(|error| {
                AuthError::FailedToGetToken {
                    message: "Fetching authentication token failed!".to_string(),
                    cause: WrappedRequestTokenError(error),
                }
            })?;

        let mut state = token_store.write().await;
        ConfidentialClient::update_storage_token(&response, &mut state)?;

        Ok(Token { value: response.access_token().secret().to_string() })
    }
}