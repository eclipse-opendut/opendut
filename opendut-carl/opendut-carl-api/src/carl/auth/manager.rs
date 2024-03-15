use chrono::{NaiveDateTime, Utc};
use oauth2::{AccessToken, AuthUrl, Scope, TokenResponse, TokenUrl};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use tokio::sync::{RwLock, RwLockWriteGuard};

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


impl TryFrom<OidcIdentityProviderConfig> for AuthenticationManager {
    type Error = anyhow::Error;

    fn try_from(idp_config: OidcIdentityProviderConfig) -> anyhow::Result<Self> {
        let auth_endpoint = idp_config.issuer_url.join("protocol/openid-connect/auth")
            .map_err(|error| anyhow::anyhow!("Failed to derive auth url from issuer url: {}", error))?;
        let token_endpoint = idp_config.issuer_url.join("protocol/openid-connect/token")
            .map_err(|error| anyhow::anyhow!("Failed to derive token url from issuer url: {}", error))?;

        let client = BasicClient::new(
            idp_config.client_id,
            Some(idp_config.client_secret),
            AuthUrl::from_url(auth_endpoint),
            Some(TokenUrl::from_url(token_endpoint)),
        );

        Ok(Self {
            client,
            scopes: idp_config.scopes,
            state: Default::default(),
        })
    }
}

impl AuthenticationManager {
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
                    cause: error.to_string(),
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

#[cfg(test)]
mod tests {
    use googletest::assert_that;
    use googletest::matchers::eq;
    use oauth2::{ClientId, ClientSecret};
    use rstest::{fixture, rstest};
    use url::Url;
    use crate::carl::auth::auth_config::OidcIdentityProviderConfig;
    use crate::carl::auth::manager::AuthenticationManager;

    #[fixture]
    fn authentication_manager() -> AuthenticationManager {
        let idp_config: OidcIdentityProviderConfig = OidcIdentityProviderConfig {
            client_id: ClientId::new("opendut-edgar-client".to_string()),
            client_secret: ClientSecret::new("c7d6ace0-b90f-471a-bb62-a4ecac4150f8".to_string()),
            issuer_url: Url::parse("http://localhost:8081/realms/opendut/").unwrap(),
            scopes: vec![],
        };

        AuthenticationManager::try_from(idp_config).unwrap()
    }

    #[rstest]
    #[tokio::test]
    #[ignore]
    async fn test_auth_manager_get_token(authentication_manager: AuthenticationManager) {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command: cargo test -- --include-ignored
         */
        let token = authentication_manager.get_token().await.unwrap();
        assert!(token.value.len() > 100);
    }


}
