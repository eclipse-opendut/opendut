use chrono::{NaiveDateTime, Utc};
use oauth2::{AccessToken, AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use tokio::sync::{RwLock, RwLockWriteGuard};

use crate::carl::auth::reqwest_client::async_http_client;
use crate::carl::OidcIdentityProviderConfig;

#[derive(Debug)]
pub struct AuthenticationManager {
    client: BasicClient,
    scopes: Vec<Scope>,

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
    FailedToGetToken { message: String, cause: String },
    #[error("ExpirationFieldMissing: {message}.")]
    ExpirationFieldMissing { message: String },
}

pub struct Token {
    pub value: String,
}


impl AuthenticationManager {
    pub fn new(oidc_identity_provider_config: OidcIdentityProviderConfig) -> Self {
        let auth_url = AuthUrl::new(
            format!("{}/protocol/openid-connect/auth", oidc_identity_provider_config.issuer_url)
        ).expect("Invalid oidc authorization endpoint URL");
        let token_url = TokenUrl::new(
            format!("{}/protocol/openid-connect/token", oidc_identity_provider_config.issuer_url)
        ).expect("Invalid oidc token endpoint URL");

        let client = BasicClient::new(
            ClientId::new(oidc_identity_provider_config.id),
            Some(ClientSecret::new(oidc_identity_provider_config.secret)),
            auth_url,
            Some(token_url),
        );
        let scopes = oidc_identity_provider_config.scopes.trim_matches('"')
            .split(',').map(|s| Scope::new(s.to_string())).collect::<Vec<_>>();

        AuthenticationManager {
            client,
            scopes,
            state: Default::default(),
        }
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
        let response = self.client.exchange_client_credentials()
            .add_scopes(self.scopes.clone())
            .request_async(async_http_client)
            .await
            .map_err(|error|
                AuthError::FailedToGetToken {
                    message: "Fetching authentication token failed!".to_string(),
                    cause: error.to_string()
                }
            )?;

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
}
