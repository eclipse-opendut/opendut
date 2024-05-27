use std::fmt::Formatter;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::{NaiveDateTime, Utc};
use config::Config;
use oauth2::{AccessToken, TokenResponse};
use oauth2::basic::{BasicClient, BasicTokenResponse};
use backoff::ExponentialBackoffBuilder;
use std::sync::{RwLock, RwLockWriteGuard};
use tokio::sync::{Mutex, TryLockError};
use tonic::{Request, Status};
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tracing::debug;
use crate::confidential::config::{ConfidentialClientConfig, ConfidentialClientConfigData};
use crate::confidential::blocking::reqwest_client::OidcBlockingReqwestClient;
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
    FailedToLockConfidentialClient { message: String, cause: backoff::Error<TryLockError> },
    
}

#[derive(Clone)]
pub struct Token {
    pub value: String,
}

#[derive(Clone)]
pub struct ConfClientArcMutex<T>(pub Arc<Mutex<T>>);

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

                let reqwest_client = OidcBlockingReqwestClient::from_config(settings).await
                    .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to create reqwest client."), cause: cause.into() })?;

                let client = ConfidentialClient::from_client_config(client_config.clone(), reqwest_client).await?;
                
                match client.check_connection(client_config) {
                    Ok(_) => { Ok(Some(client)) }
                    Err(error) => { Err(error) }
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

    fn check_connection(&self, idp_config: ConfidentialClientConfigData) -> Result<(), ConfidentialClientError> {

        let token_endpoint = idp_config.issuer_url.join("protocol/openid-connect/token")
            .map_err(|error| ConfidentialClientError::UrlParse { message: String::from("Failed to derive token url from issuer url: "), cause: error })?;
        
        let exponential_backoff = ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(Some(Duration::from_secs(120)))
            .build();

        let operation = || {
            self.reqwest_client.client.get(token_endpoint.clone()).send()?;
            Ok(())
        };
        
        let backoff_result = backoff::retry(exponential_backoff, operation);

        match backoff_result {
            Ok(_) => { Ok(()) }
            Err(error) => {
                Err(ConfidentialClientError::KeycloakConnection { message: String::from("Could not connect to keycloak"), cause: error })
            }
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

impl Interceptor for ConfClientArcMutex<Option<ConfidentialClientRef>> {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {

        let cloned_arc_mutex = Arc::clone(&self.0);
        
        let exponential_backoff = ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(Some(Duration::from_secs(120)))
            .build();

        let operation = || {
            let mutex_guard = cloned_arc_mutex.try_lock()?;
            Ok(mutex_guard)
        };

        let backoff_result = backoff::retry(exponential_backoff, operation);
        
        let token = match backoff_result {
            Ok(mutex_guard) => { 
                let confidential_client= mutex_guard.clone();
                confidential_client.map(|client| client.get_token())
            }
            Err(error) => {
                eprintln!("Failed to acquire lock on the Confidential Client definitively. The following telemetry data will not be transmitted.");
                eprintln!("Failed request: {:?}", request);
                Some(Err(AuthError::FailedToLockConfidentialClient {message: "Unable to acquire lock on the Confidential Client".to_owned(), cause: error}))
            }
        };
        
        return match token {
            None => { Ok(request) }
            Some(token_result) => {
                match token_result {
                    Ok(token) => {
                        let token_string = token.value.as_str();
                        let bearer_header = format!("Bearer {token_string}");
                        request.metadata_mut().insert(http::header::AUTHORIZATION.as_str(), MetadataValue::from_str(&bearer_header).expect("Cannot create metadata value from bearer header"));
                        Ok(request)
                    }
                    Err(error) => { Err(Status::unauthenticated(format!("{}", error))) }
                }
            }
        };
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
    use crate::confidential::blocking;

    #[test]
    #[ignore]
    fn test_get_token_example() {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command:
         * cargo test --package opendut-auth --all-features -- --include-ignored
         */
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
        let reqwest_client = blocking::reqwest_client::OidcBlockingReqwestClient::from_pem(certificate).unwrap();
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
