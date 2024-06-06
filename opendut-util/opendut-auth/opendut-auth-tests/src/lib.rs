use anyhow::anyhow;
use oauth2::{ClientId, ClientSecret, RedirectUrl};
use openidconnect::RegistrationUrl;
use pem::Pem;
use rstest::fixture;
use url::Url;

use opendut_auth::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use opendut_auth::confidential::config::ConfidentialClientConfigData;
use opendut_auth::confidential::pem::PemFromConfig;
use opendut_auth::confidential::reqwest_client::OidcReqwestClient;
use opendut_auth::registration::client::{DEVICE_REDIRECT_URL, RegistrationClient, RegistrationClientRef};
use opendut_auth::registration::config::RegistrationClientConfig;
use opendut_auth::registration::resources::ResourceHomeUrl;
use opendut_util_core::project;

#[fixture]
pub async fn issuer_certificate_authority() -> Pem {
    Pem::from_file_path("resources/development/tls/insecure-development-ca.pem").await
        .expect("Failed to resolve development ca in resources directory.")
}

#[fixture]
pub async fn confidential_carl_client() -> ConfidentialClientRef {
    let issuer_url = "https://keycloak/realms/opendut/".to_string();  // This is the URL for the keycloak server in the test environment
    //let issuer_url = "http://192.168.56.10:8081/realms/opendut/".to_string();  // This is the URL for the keycloak server in the test environment (valid in host system and opendut-vm)

    let client_config = ConfidentialClientConfigData::new(
        ClientId::new("opendut-carl-client".to_string()),
        ClientSecret::new("6754d533-9442-4ee6-952a-97e332eca38e".to_string()),
        Url::parse(&issuer_url).unwrap(),
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

#[fixture]
pub async fn registration_client(#[future] confidential_carl_client: ConfidentialClientRef) -> RegistrationClientRef {
    /*
     * Issuer URL for keycloak needs to align with FRONTEND_URL in Keycloak realm setting.
     * Localhost address is always fine, though.
     */
    let issuer_remote_url_string = "https://keycloak/realms/opendut/".to_string();  // works inside OpenDuT-VM
    let issuer_remote_url = Url::parse(&issuer_remote_url_string).unwrap();
    let carl_idp_config = RegistrationClientConfig {
        issuer_remote_url: issuer_remote_url.clone(),
        peer_credentials: None,
        device_redirect_url: RedirectUrl::new(DEVICE_REDIRECT_URL.to_string()).unwrap(),
        client_home_base_url: ResourceHomeUrl::new(Url::parse("https://carl/resources/uuid-123").unwrap()),
        registration_url: RegistrationUrl::from_url(issuer_remote_url.join("clients-registrations/openid-connect").unwrap()),
    };
    let client = confidential_carl_client.await;
    RegistrationClient::new(carl_idp_config, client)
}

#[cfg(test)]
mod auth_tests {
    use googletest::assert_that;
    use googletest::matchers::eq;
    use http::{HeaderMap, HeaderValue};
    use oauth2::HttpRequest;
    use pem::Pem;
    use rstest::rstest;

    use opendut_auth::confidential::reqwest_client::OidcReqwestClient;
    use opendut_auth::registration::client::{RegistrationClientError, RegistrationClientRef};
    use opendut_types::resources::Id;

    use crate::{issuer_certificate_authority, registration_client};

    async fn delete_client(client: RegistrationClientRef, delete_client_id: String, issuer_ca: Pem) -> Result<(), RegistrationClientError> {
        let client_id = client.inner.config.client_id.clone().to_string();
        let access_token = client.inner.get_token().await
            .map_err(|error| RegistrationClientError::RequestError { error: format!("Could not fetch token to delete client {}!", client_id), cause: error.into() })?;
        let delete_client_url = client.inner.config.issuer_url.join("/admin/realms/opendut/clients/").unwrap().join(&delete_client_id.to_string())
            .map_err(|error| RegistrationClientError::InvalidConfiguration { error: format!("Invalid client URL: {}", error) })?;

        let mut headers = HeaderMap::new();
        let bearer_header = format!("Bearer {}", access_token.to_string());
        let access_token_value = HeaderValue::from_str(&bearer_header)
            .map_err(|error| RegistrationClientError::InvalidConfiguration { error: error.to_string() })?;
        headers.insert(http::header::AUTHORIZATION, access_token_value);

        let request = HttpRequest {
            method: http::Method::DELETE,
            url: delete_client_url,
            headers,
            body: vec![],
        };

        let reqwest_client = OidcReqwestClient::from_pem(issuer_ca)
            .map_err(|error| RegistrationClientError::InvalidConfiguration { error: format!("Failed to load certificate authority. {}", error) })?;

        let response = reqwest_client.async_http_client(request)
            .await
            .map_err(|error| RegistrationClientError::RequestError { error: "OIDC client delete request failed!".to_string(), cause: Box::new(error) })?;
        assert_eq!(response.status_code, 204, "Failed to delete client with id '{:?}': {:?}", client_id, response.body);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore]
    async fn test_register_new_oidc_client(#[future] registration_client: RegistrationClientRef, #[future] issuer_certificate_authority: Pem) {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command: cargo test -- --include-ignored
         */
        let client = registration_client.await;
        let pem = issuer_certificate_authority.await;
        println!("{:?}", client);
        let resource_id = Id::random();
        let credentials = client.register_new_client(resource_id).await.unwrap();
        let (client_id, client_secret) = (credentials.client_id.value(), credentials.client_secret.value());
        println!("New client id: {}, secret: {}", client_id, client_secret);
        delete_client(client, client_id.clone(), pem).await.unwrap();
        assert_that!(client_id.len().gt(&10), eq(true));
    }

}
