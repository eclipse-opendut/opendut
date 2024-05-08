use std::sync::Arc;

use config::Config;
use openidconnect::{ClientName, ClientUrl};
use openidconnect::core::{CoreClientRegistrationRequest, CoreGrantType};
use openidconnect::registration::EmptyAdditionalClientMetadata;

use opendut_types::resources::Id;
use opendut_types::util::net::{ClientCredentials, ClientId, ClientSecret};

use crate::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use crate::registration::config::RegistrationClientConfig;

pub type RegistrationClientRef = Arc<RegistrationClient>;

pub const DEVICE_REDIRECT_URL: &str = "http://localhost:12345/device";

#[derive(Debug)]
pub struct RegistrationClient {
    pub inner: ConfidentialClientRef,
    pub config: RegistrationClientConfig,
}

#[derive(thiserror::Error, Debug)]
pub enum RegistrationClientError {
    #[error("Invalid configuration:\n  {error}")]
    InvalidConfiguration {
        error: String,
    },
    #[error("Failed request:\n {error}")]
    RequestError {
        error: String,
        inner: Box<dyn std::error::Error + Send + Sync>,  // RequestTokenError<OidcClientError<reqwest::Error>, BasicErrorResponse>
    },
    #[error("Failed to register new client:\n  {error}")]
    Registration {
        error: String,
    },
}


impl RegistrationClient {
    pub async fn from_settings(settings: &Config) -> Result<Option<RegistrationClientRef>, RegistrationClientError> {
        let inner = ConfidentialClient::from_settings(settings).await
            .map_err(|error| RegistrationClientError::InvalidConfiguration { error: error.to_string() })?;
        match inner {
            None => {
                // Authentication is disabled, ergo no registration client is needed
                Ok(None)
            }
            Some(inner) => {
                let registration_config = RegistrationClientConfig::from_settings(settings, &inner)?;

                Ok(Some(Self::new(registration_config, inner)))
            }
        }
    }
    
    pub fn new(registration_client_config: RegistrationClientConfig, inner: ConfidentialClientRef) -> RegistrationClientRef {
        Arc::new(Self {
            inner,
            config: registration_client_config,
        })
    }

    /*pub async fn from_config(config: RegistrationClientConfig) -> Result<Option<RegistrationClientRef>, RegistrationClientError> {
        
    }*/

    pub async fn register_new_client(&self, resource_id: Id) -> Result<ClientCredentials, RegistrationClientError> {
        match self.config.peer_credentials.clone() {
            Some(peer_credentials) => {
                Ok(peer_credentials)
            }
            None => {
                let access_token = self.inner.get_token().await
                    .map_err(|error| RegistrationClientError::RequestError { error: error.to_string(), inner: error.into() })?;
                let additional_metadata = EmptyAdditionalClientMetadata {};
                let redirect_uris = vec![self.config.device_redirect_url.clone()];
                let grant_types = vec![CoreGrantType::ClientCredentials];
                let request: CoreClientRegistrationRequest =
                    openidconnect::registration::ClientRegistrationRequest::new(redirect_uris, additional_metadata)
                        .set_grant_types(Some(grant_types));
                let registration_url = self.config.registration_url.clone();

                let client_name: ClientName = ClientName::new(resource_id.to_string());
                let resource_uri = self.config.client_home_base_url.resource_url(resource_id)
                    .map_err(|error| RegistrationClientError::Registration {
                        error: format!("Failed to forge client url: {:?}", error),
                    })?;
                let client_home_uri = ClientUrl::new(String::from(resource_uri))
                    .map_err(|error| RegistrationClientError::Registration {
                        error: format!("Failed to forge client url: {:?}", error),
                    })?;
                let response = request
                    .set_initial_access_token(Some(access_token.oauth_token()))
                    .set_client_name(Some(
                        vec![(None, client_name)]
                            .into_iter()
                            .collect(),
                    ))
                    .set_client_uri(Some(vec![(None, client_home_uri)]
                        .into_iter()
                        .collect()))
                    .register_async(&registration_url, move |request| {
                        self.inner.reqwest_client.async_http_client(request)
                    }).await;
                match response {
                    Ok(response) => {
                        let client_id = response.client_id();
                        let client_secret = response.client_secret().expect("Confidential client required!");

                        Ok(ClientCredentials {
                            client_id: ClientId(client_id.to_string()),
                            client_secret: ClientSecret(client_secret.secret().to_string()),
                        })
                    }
                    Err(error) => {
                        Err(RegistrationClientError::Registration {
                            error: format!("{:?}", error),
                        })
                    }
                }
            }
        }
    }
}