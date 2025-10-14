use const_format::formatcp;
use config::Config;
use oauth2::RedirectUrl;
use openidconnect::RegistrationUrl;
use opendut_model::util::net::{ClientCredentials, ClientId, ClientSecret};
use crate::confidential::client::ConfidentialClient;
use crate::confidential::IssuerUrl;
use crate::registration::client::{DEVICE_REDIRECT_URL, RegistrationClientError};
use crate::registration::resources::ResourceHomeUrl;

#[derive(Debug)]
pub struct RegistrationClientConfig {
    pub peer_credentials: Option<ClientCredentials>,
    pub device_redirect_url: RedirectUrl,
    pub client_home_base_url: ResourceHomeUrl,
    pub registration_url: RegistrationUrl,
    pub issuer_url: IssuerUrl,
    pub issuer_remote_url: IssuerUrl,
    pub issuer_admin_url: IssuerUrl,
}

pub(crate) const AUTH_CLIENT_CONFIG_PREFIX: &str = "network.oidc.client";

impl RegistrationClientConfig {
    const ISSUER_REMOTE_URL: &'static str = formatcp!("{AUTH_CLIENT_CONFIG_PREFIX}.issuer.remote.url");
    /*
     Administrative API endpoint of the OIDC issuer (keycloak)
     
     https://www.keycloak.org/docs-api/latest/rest-api/index.html
     
     # List clients
     GET /admin/realms/{realm}/clients/
     
     # Delete the client
     DELETE /admin/realms/{realm}/clients/{client-uuid}
     */
    
    const ISSUER_ADMIN_URL: &'static str = formatcp!("{AUTH_CLIENT_CONFIG_PREFIX}.issuer.admin.url");
    const COMMON_PEER_ID: &'static str = formatcp!("{AUTH_CLIENT_CONFIG_PREFIX}.peer.id");
    const COMMON_PEER_SECRET: &'static str = formatcp!("{AUTH_CLIENT_CONFIG_PREFIX}.peer.secret");

    pub fn from_settings(settings: &Config, client: &ConfidentialClient) -> Result<Self, RegistrationClientError> {
        let device_redirect_url = RedirectUrl::new(DEVICE_REDIRECT_URL.to_string())
            .map_err(|error| RegistrationClientError::InvalidConfiguration { error: format!("Failed to parse device redirect URL: {error}") })?;
        let client_home_base_url = ResourceHomeUrl::try_from(settings)
            .map_err(|error| RegistrationClientError::InvalidConfiguration { error: format!("Failed to load client home base URL: {error}") })?;
        let registration_url = RegistrationUrl::from_url(
            client.issuer_url.value().join("clients-registrations/openid-connect")
                .map_err(|error| RegistrationClientError::InvalidConfiguration { error: format!("Invalid registration endpoint URL: {error}") })?
        );
        let issuer_remote_url = settings.get_string(RegistrationClientConfig::ISSUER_REMOTE_URL)
            .map_err(|error| RegistrationClientError::InvalidConfiguration { 
                error: format!("Failed to load registration URL from config field {}: {}", RegistrationClientConfig::ISSUER_REMOTE_URL, error) 
            })?;
        let issuer_remote_url = IssuerUrl::try_from(issuer_remote_url.as_str())
            .map_err(|error| RegistrationClientError::InvalidConfiguration {
                error: format!("Failed to parse issuer remote URL: {error}") 
            })?;
        let issuer_admin_url = settings.get_string(RegistrationClientConfig::ISSUER_ADMIN_URL)
            .map_err(|error| RegistrationClientError::InvalidConfiguration {
                error: format!("Failed to load registration URL from config field {}: {}", RegistrationClientConfig::ISSUER_ADMIN_URL, error)
            })?;
        let issuer_admin_url = IssuerUrl::try_from(issuer_admin_url.as_str())
            .map_err(|error| RegistrationClientError::InvalidConfiguration {
                error: format!("Failed to parse issuer admin URL: {error}")
            })?;
        
        let peer_id = settings.get_string(RegistrationClientConfig::COMMON_PEER_ID).ok();
        let peer_secret = settings.get_string(RegistrationClientConfig::COMMON_PEER_SECRET).ok();
        let peer_credentials = match (peer_id, peer_secret) {
            (Some(id), Some(secret)) => {
                Some(ClientCredentials {
                    client_id: ClientId(id),
                    client_secret: ClientSecret(secret),
                })
            }
            _ => None
        };

        Ok(Self {
            peer_credentials,
            device_redirect_url,
            client_home_base_url,
            registration_url,
            issuer_url: client.issuer_url.clone(),
            issuer_remote_url,
            issuer_admin_url,
        })
    }
    
    pub fn new(peer_credentials: Option<ClientCredentials>, device_redirect_url: RedirectUrl, client_home_base_url: ResourceHomeUrl, 
               registration_url: RegistrationUrl, issuer_url: IssuerUrl, issuer_remote_url: IssuerUrl, issuer_admin_url: IssuerUrl) -> Self {
        Self {
            peer_credentials,
            device_redirect_url,
            client_home_base_url,
            registration_url,
            issuer_url,
            issuer_remote_url,
            issuer_admin_url,
        }
    }
}
