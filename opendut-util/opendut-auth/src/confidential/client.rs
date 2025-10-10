use std::fmt::Formatter;
use std::ops::Sub;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use backon::BlockingRetryable;
use chrono::{NaiveDateTime, Utc};
use config::Config;
use oauth2::basic::BasicTokenResponse;
use oauth2::{AccessToken, TokenResponse};
use tokio::sync::{Mutex, RwLock, RwLockWriteGuard, TryLockError};
use tonic::{Request, Status};
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tracing::debug;
use backon::Retryable;
use reqwest_middleware::ClientWithMiddleware;
use tokio::runtime::Handle;
use crate::confidential::config::{ConfidentialClientConfig, ConfidentialClientConfigData, ConfiguredClient};
use crate::confidential::error::{ConfidentialClientError, WrappedRequestTokenError};
use crate::confidential::reqwest_client::OidcReqwestClient;
use opendut_util_core::future::ExplicitSendFutureWrapper;
use crate::confidential::middleware::OAuthMiddleware;
use crate::TOKEN_GRACE_PERIOD;

#[derive(Debug)]
pub struct ConfidentialClient {
    inner: ConfiguredClient,
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
    #[error("FailedToUpdateToken: {message} cause: {cause}.")]
    FailedToLockConfidentialClient { message: String, cause: TryLockError },
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
                debug!("OIDC configuration loaded: client_id='{}', issuer_url='{}'", client_config.client_id.as_str(), client_config.issuer_url.as_str());
                let reqwest_client = OidcReqwestClient::from_config(settings).await
                    .map_err(|cause| ConfidentialClientError::Configuration { message: String::from("Failed to create reqwest client."), cause: cause.into() })?;
                let client = ConfidentialClient::from_client_config(client_config.clone(), reqwest_client)?;

                match client.check_connection(client_config).await {
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

    pub fn from_client_config(client_config: ConfidentialClientConfigData, reqwest_client: OidcReqwestClient) -> Result<ConfidentialClientRef, ConfidentialClientError> {
        let inner = client_config.get_client()?;

        let client = Self {
            inner,
            reqwest_client,
            config: client_config,
            state: Default::default(),
        };
        Ok(Arc::new(client))
    }
    async fn check_connection(&self, idp_config: ConfidentialClientConfigData) -> Result<(), ConfidentialClientError> {

        let token_endpoint = idp_config.issuer_url.join("protocol/openid-connect/token")
            .map_err(|error| ConfidentialClientError::UrlParse { message: String::from("Failed to derive token url from issuer url: "), cause: error })?;

        let operation = move || {
            let client = self.reqwest_client.client.clone();
            let token_endpoint = token_endpoint.clone();
            async move {
                client.get(token_endpoint.clone()).send().await
            }
        };

        let backoff_result = operation
            .retry(
                backon::ExponentialBuilder::default()
                    .with_max_delay(Duration::from_secs(120))
            )
            .notify(|err: &reqwest::Error, dur: Duration| {
                eprintln!("Retrying connection to issuer. {err:?} after {dur:?}");
            })
            .await;

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

    async fn fetch_token(&self) -> Result<Token, AuthError> {
        let response = ExplicitSendFutureWrapper::from(
                self.inner.exchange_client_credentials()
                    .add_scopes(self.config.scopes.clone())
                    .request_async(&|request| { self.reqwest_client.async_http_client(request) })
            ).await
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

    pub fn build_client_with_middleware(confidential_client: ConfidentialClientRef) -> ClientWithMiddleware {
        let inner = confidential_client.reqwest_client.client();
        reqwest_middleware::ClientBuilder::new(inner)
            .with(OAuthMiddleware::new(confidential_client))
            .build()
    }

    pub async fn get_token(&self) -> Result<Token, AuthError> {
        let token_storage = self.state.read().await.clone();
        let access_token = match token_storage {
            None => {
                self.fetch_token().await?
            }
            Some(token) => {
                if Utc::now().naive_utc().lt(&token.expires_in.sub(TOKEN_GRACE_PERIOD)) {
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

#[derive(Clone)]
pub struct ConfClientArcMutex<T: Clone + Send + Sync + 'static> {
    pub mutex: Arc<Mutex<T>>,
    pub handle: Handle,
}

impl Interceptor for ConfClientArcMutex<Option<ConfidentialClientRef>> {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {

        let cloned_arc_mutex = Arc::clone(&self.mutex);

        let operation = || {
            let mutex_guard = cloned_arc_mutex.try_lock()?;
            match mutex_guard.as_ref() {
                Some(confidential_client) => {
                    let token = self.handle.block_on(async move {
                        confidential_client.get_token().await
                    });
                    Ok(Some(token))
                }
                None => {
                    Ok(None)
                }
            }
        };

        let mut retries = 0;
        let start = Instant::now();
        let backoff_result = operation
            .retry(
                backon::ExponentialBuilder::default()
                    .with_max_delay(Duration::from_secs(120))
            )
            .notify(|_, dur: Duration| {
                retries += 1;
                eprintln!("Failed to acquire lock on confidential client in telemetry request interceptor. Retrying to get access token after {dur:?}.");
            })
            .call();

        let token = match backoff_result {
            Ok(token) => {
                if retries > 0 {
                    let duration = Instant::now().saturating_duration_since(start);
                    eprintln!("Acquired lock on confidential client after <{}> retries and <{}> seconds.", retries, duration.as_secs());
                }
                token
            }
            Err(error) => {
                eprintln!("Failed to acquire lock on the Confidential Client definitively. The following telemetry request will not be transmitted.");
                eprintln!("Failed request: {request:?}");
                Some(Err(AuthError::FailedToLockConfidentialClient { message: "Unable to acquire lock on the Confidential Client".to_owned(), cause: error }))
            }
        };

        match token {
            None => { Ok(request) }
            Some(token_result) => {
                match token_result {
                    Ok(token) => {
                        let token_string = token.value.as_str();
                        let bearer_header = format!("Bearer {token_string}");
                        request.metadata_mut().insert(http::header::AUTHORIZATION.as_str(), MetadataValue::from_str(&bearer_header).expect("Cannot create metadata value from bearer header"));
                        Ok(request)
                    }
                    Err(error) => { Err(Status::unauthenticated(format!("{error}"))) }
                }
            }
        }
    }
}
