use oauth2::{AccessToken, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenResponse, TokenUrl};
use oauth2::basic::BasicClient;
use oauth2::reqwest::http_client;
use openidconnect::core::{CoreClientRegistrationRequest, CoreGrantType};
use openidconnect::registration::EmptyAdditionalClientMetadata;
use openidconnect::RegistrationUrl;

pub const DEVICE_REDIRECT_URL: &str = "http://localhost:12345/device";

#[derive(Debug)]
pub struct AuthManager {
    client: BasicClient,
    registration_url: RegistrationUrl,
    device_redirect_url: RedirectUrl,
}

#[derive(Debug)]
pub struct ClientCredentials {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthManagerError {
    #[error("Invalid configuration:\n  {error}")]
    InvalidConfiguration {
        error: String,
    },
    #[error("Invalid client credentials:\n  {error}")]
    InvalidCredentials {
        error: String,
    },
    #[error("Failed to register new peer:\n  {error}")]
    Registration {
        error: String,
    },
}

impl AuthManager {
    /// issuer_url for keycloak includes realm name: http://localhost:8081/realms/opendut
    pub fn new(client_id: String, client_secret: String, issuer_url: String) -> Result<Self, AuthManagerError> {
        let auth_url = format!("{}/protocol/openid-connect/auth", issuer_url);
        let token_url = format!("{}/protocol/openid-connect/token", issuer_url);
        let registration_url = format!("{}/clients-registrations/openid-connect", issuer_url);

        let auth_url = AuthUrl::new(auth_url)
            .map_err(|error| AuthManagerError::InvalidConfiguration { error: format!("Invalid auth endpoint url: {}", error) })?;
        let token_url = TokenUrl::new(token_url)
            .map_err(|error| AuthManagerError::InvalidConfiguration { error: format!("Invalid token endpoint URL: {}", error) })?;
        let registration_url = RegistrationUrl::new(registration_url)
            .map_err(|error| AuthManagerError::InvalidConfiguration { error: format!("Invalid registration endpoint URL: {}", error) })?;

        let device_redirect_url = RedirectUrl::new(DEVICE_REDIRECT_URL.to_string()).expect("Could not form redirect url");

        let client_id = ClientId::new(client_id);
        let client_secret = ClientSecret::new(client_secret);
        let client =
            BasicClient::new(
                client_id,
                Some(client_secret),
                auth_url,
                Some(token_url),
            );

        Ok(AuthManager {
            client,
            registration_url,
            device_redirect_url,
        })
    }

    fn get_token(&self) -> Result<AccessToken, AuthManagerError> {
        let response = self.client.exchange_client_credentials()
            .request(http_client)
            .map_err(|error| AuthManagerError::InvalidCredentials { error: error.to_string() })?;
        Ok(response.access_token().clone())
    }

    pub fn register_new_client(&self) -> Result<ClientCredentials, AuthManagerError> {
        let access_token = self.get_token()?;
        let additional_metadata = EmptyAdditionalClientMetadata {};
        let redirect_uris = vec![self.device_redirect_url.clone()];
        let grant_types = vec![CoreGrantType::ClientCredentials];
        let request: CoreClientRegistrationRequest =
            openidconnect::registration::ClientRegistrationRequest::new(redirect_uris, additional_metadata)
                .set_grant_types(Some(grant_types));
        let registration_url = self.registration_url.clone();
        let response = request
            .set_initial_access_token(Some(access_token))
            .register(&registration_url, http_client);

        match response {
            Ok(response) => {
                let client_id = response.client_id();
                let client_secret = response.client_secret().expect("Confidential client required!");

                Ok(ClientCredentials {
                    client_id: client_id.clone(),
                    client_secret: client_secret.clone(),
                })
            }
            Err(error) => {
                Err(AuthManagerError::Registration {
                    error: format!("{:?}", error),
                })
            }
        }

    }
}


#[cfg(test)]
mod tests {
    use http::{HeaderMap, HeaderValue};
    use oauth2::HttpRequest;
    use url::Url;
    use super::*;

    fn delete_client(manager: AuthManager, client_id: &ClientId) -> Result<(), AuthManagerError> {
        let access_token = manager.get_token()?;
        let request_base_url: Url = "http://localhost:8081/admin/realms/opendut/clients/".parse().unwrap();
        let delete_client_url = request_base_url.join(&format!("{}", client_id.to_string()))
            .map_err(|error| AuthManagerError::InvalidConfiguration { error: format!("Invalid client URL: {}", error.to_string()) })?;

        let mut headers = HeaderMap::new();
        let bearer_header = format!("Bearer {}", access_token.secret().as_str());
        let access_token_value = HeaderValue::from_str(&bearer_header)
            .map_err(|error| AuthManagerError::InvalidConfiguration { error: error.to_string() })?;
        headers.insert(http::header::AUTHORIZATION, access_token_value);

        let request = HttpRequest {
            method: http::Method::DELETE,
            url: delete_client_url,
            headers,
            body: vec![],
        };
        let response = http_client(request)
            .map_err(|error| AuthManagerError::Registration { error: error.to_string() })?;
        assert_eq!(response.status_code, 204, "Failed to delete client with id '{:?}': {:?}", client_id, response.body);

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_register_new_oidc_client() {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         */
        let client_id = "opendut-carl-client".to_string();
        let client_secret = "6754d533-9442-4ee6-952a-97e332eca38e".to_string();
        let issuer_url = "http://localhost:8081/realms/opendut".to_string();
        let auth_manager = AuthManager::new(client_id, client_secret, issuer_url).unwrap();
        let credentials = auth_manager.register_new_client().unwrap();
        println!("New client id: {}, secret: {}", credentials.client_id.to_string(), credentials.client_secret.secret().to_string());
        delete_client(auth_manager, &credentials.client_id).unwrap();
    }
}
