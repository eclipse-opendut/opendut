#[cfg(test)]
mod client;

use anyhow::anyhow;
use oauth2::{ClientId, ClientSecret, RedirectUrl};
use opendut_util_core::expect_env_var;
use openidconnect::RegistrationUrl;
use pem::Pem;
use rstest::fixture;
use url::Url;

use opendut_auth::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use opendut_auth::confidential::config::ConfidentialClientConfigData;
use opendut_auth::confidential::pem::PemFromConfig;
use opendut_auth::confidential::reqwest_client::OidcReqwestClient;
use opendut_auth::registration::client::{
    DEVICE_REDIRECT_URL, RegistrationClient, RegistrationClientRef,
};
use opendut_auth::registration::config::RegistrationClientConfig;
use opendut_auth::registration::resources::ResourceHomeUrl;
use opendut_util_core::project;

#[fixture]
pub async fn localenv_reqwest_client() -> OidcReqwestClient {
    let ca_path =
        project::make_path_absolute(".ci/deploy/localenv/data/secrets/pki/opendut-ca.pem")
            .expect("Could not resolve localenv CA")
            .into_os_string()
            .into_string()
            .unwrap();
    let pem = <Pem as PemFromConfig>::from_file_path(&ca_path)
        .await
        .expect("Could not load localenv CA");
    OidcReqwestClient::from_pem(pem)
        .map_err(|cause| anyhow!("Failed to create reqwest client. Error: {}", cause))
        .unwrap()
}

#[fixture]
pub async fn confidential_carl_client(
    #[future] localenv_reqwest_client: OidcReqwestClient,
) -> ConfidentialClientRef {
    opendut_util_core::testing::init_localenv_secrets();
    let issuer_url = "https://auth.opendut.local/realms/opendut/".to_string(); // This is the URL for the keycloak server in the test environment

    let client_config = ConfidentialClientConfigData::new(
        ClientId::new("opendut-carl-client".to_string()),
        ClientSecret::new(expect_env_var("OPENDUT_CARL_NETWORK_OIDC_CLIENT_SECRET")),
        Url::parse(&issuer_url).unwrap(),
        vec![],
    );
    let reqwest_client = localenv_reqwest_client.await;

    ConfidentialClient::from_client_config(client_config, reqwest_client)
        .await
        .unwrap()
}

#[fixture]
pub async fn registration_client(
    #[future] confidential_carl_client: ConfidentialClientRef,
) -> RegistrationClientRef {
    /*
     * Issuer URL for keycloak needs to align with FRONTEND_URL in Keycloak realm setting.
     * Localhost address is always fine, though.
     */
    let issuer_remote_url_string = "https://auth.opendut.local/realms/opendut/".to_string(); // works inside OpenDuT-VM
    let issuer_remote_url = Url::parse(&issuer_remote_url_string).unwrap();
    let carl_idp_config = RegistrationClientConfig {
        issuer_remote_url: issuer_remote_url.clone(),
        peer_credentials: None,
        device_redirect_url: RedirectUrl::new(DEVICE_REDIRECT_URL.to_string()).unwrap(),
        client_home_base_url: ResourceHomeUrl::new(
            Url::parse("https://carl.opendut.local/resources/uuid-123").unwrap(),
        ),
        registration_url: RegistrationUrl::from_url(
            issuer_remote_url
                .join("clients-registrations/openid-connect")
                .unwrap(),
        ),
        issuer_admin_url: issuer_remote_url
            .join("https://auth.opendut.local/admin/realms/opendut/")
            .unwrap(),
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
    use opendut_auth::registration::resources::UserId;
    use opendut_model::resources::Id;

    use crate::registration_client;

    #[test_with::env(OPENDUT_RUN_KEYCLOAK_INTEGRATION_TESTS)]
    #[rstest]
    #[tokio::test]
    async fn test_register_new_oidc_client(#[future] registration_client: RegistrationClientRef) {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command in opendut-vm:
         * export OPENDUT_RUN_KEYCLOAK_INTEGRATION_TESTS=true
         * cargo test -- --include-ignored
         */
        let client: RegistrationClientRef = registration_client.await;
        println!("{client:?}");
        let resource_id = Id::random();
        let user_id = UserId {
            value: String::from("deleteTest"),
        };
        let credentials = client
            .register_new_client_for_user(resource_id, user_id)
            .await
            .unwrap();
        let (client_id, client_secret) = (
            credentials.client_id.value(),
            credentials.client_secret.value(),
        );
        assert_that!(client_id.len().gt(&10), eq(true));
        println!("New client id: {client_id}, secret: {client_secret}");

        let client_list: Clients = client.list_clients().await.unwrap();
        let filtered_client_list = client_list.filter_clients_by_resource_id(resource_id);
        assert!(!filtered_client_list.is_empty());

        client
            .delete_client_by_resource_id(resource_id)
            .await
            .unwrap();

        let client_list: Clients = client.list_clients().await.unwrap();
        let filtered_client_list = client_list.filter_clients_by_resource_id(resource_id);
        assert!(filtered_client_list.is_empty());
    }

    #[test_with::env(OPENDUT_RUN_KEYCLOAK_INTEGRATION_TESTS)]
    #[rstest]
    #[tokio::test]
    async fn test_register_oidc_client_twice(#[future] registration_client: RegistrationClientRef) {
        const EXPECTED_CLIENT_COUNT: usize = 1;

        let client: RegistrationClientRef = registration_client.await;
        let resource_id = Id::random();
        let user_id = UserId {
            value: String::from("deleteTest"),
        };
        let _credentials1 = client
            .register_new_client_for_user(resource_id, user_id.clone())
            .await
            .unwrap();
        let _credentials2 = client
            .register_new_client_for_user(resource_id, user_id)
            .await
            .unwrap();
        let client_list: Clients = client.list_clients().await.unwrap();
        let filtered_client_list = client_list.filter_clients_by_resource_id(resource_id);
        println!("Clients: {filtered_client_list:?}");
        let client_count = filtered_client_list.len();
        assert_eq!(client_count, EXPECTED_CLIENT_COUNT);

        client
            .delete_client_by_resource_id(resource_id)
            .await
            .unwrap();
        let client_list: Clients = client.list_clients().await.unwrap();
        let filtered_client_list = client_list.filter_clients_by_resource_id(resource_id);
        assert!(filtered_client_list.is_empty());
    }

    /*
    use opendut_auth::registration::resources::ResourceHomeUrl;

    #[rstest]
    #[tokio::test]
    #[ignore]
    async fn delete_all_oidc_clients(#[future] registration_client: RegistrationClientRef) {
        /*
         * This test is ignored because it requires a running keycloak server from the test environment.
         * To run this test, execute the following command: cargo test -- --include-ignored
         */
        let registration_client: RegistrationClientRef = registration_client.await;
        println!("{:?}", registration_client);
        let client_list: Clients = registration_client.list_clients().await.unwrap();
        let path = ResourceHomeUrl::new(registration_client.config.client_home_base_url.value().join("/resources/").unwrap());
        let filtered_client_list = client_list.filter_carl_clients(&path);

        let necessary_client_list_length = client_list.value().len() - filtered_client_list.len();

        for client in filtered_client_list {
            registration_client.delete_client(&client.client_id).await.unwrap();
        }

        let client_list_after_deletion: Clients = registration_client.list_clients().await.unwrap();
        assert_eq!(necessary_client_list_length, client_list_after_deletion.value().len());
    }
    */
}
