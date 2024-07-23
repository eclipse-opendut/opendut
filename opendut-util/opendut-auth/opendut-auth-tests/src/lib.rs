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
        issuer_admin_url: issuer_remote_url.join("https://keycloak/admin/realms/").unwrap(),
    };
    let client = confidential_carl_client.await;
    RegistrationClient::new(carl_idp_config, client)
}

#[cfg(test)]
mod auth_tests {
    use googletest::assert_that;
    use googletest::matchers::eq;
    use rstest::rstest;
    
    use opendut_auth::registration::client::{Clients, RegistrationClientRef};
    use opendut_types::resources::Id;

    use crate::{registration_client};

    #[rstest]
    #[tokio::test]
    #[ignore]
    async fn test_register_new_oidc_client(#[future] registration_client: RegistrationClientRef) {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command: cargo test -- --include-ignored
         */
        let client: RegistrationClientRef = registration_client.await;
        println!("{:?}", client);
        let resource_id = Id::random();
        let user_id = String::from("deleteTest");
        let credentials = client.register_new_client_for_user(resource_id, user_id.clone()).await.unwrap();
        let (client_id, client_secret) = (credentials.client_id.value(), credentials.client_secret.value());
        assert_that!(client_id.len().gt(&10), eq(true));
        println!("New client id: {}, secret: {}", client_id, client_secret);
        let client_list: Clients = client.list_clients(resource_id).await.unwrap();
        assert!(!client_list.value().is_empty());
        client.delete_client(resource_id).await.unwrap();
        let client_list = client.list_clients(resource_id).await.unwrap();
        assert!(client_list.value().is_empty());
    }
}
