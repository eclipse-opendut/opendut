use oauth2::{ClientId, ClientSecret};
use rstest::{fixture, rstest};
use serde::Deserialize;
use opendut_auth::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use opendut_auth::confidential::config::{OidcClientConfig, OidcConfidentialClientConfig};
use opendut_auth::confidential::IssuerUrl;
use opendut_auth::confidential::reqwest_client::Client as ReqwestClient;
use crate::localenv_reqwest_client;

#[fixture]
async fn confidential_edgar_client(#[future] localenv_reqwest_client: ReqwestClient) -> ConfidentialClientRef {
    opendut_util_core::testing::init_localenv_secrets();
    let client_config = OidcClientConfig::Confidential(OidcConfidentialClientConfig::new(
        ClientId::new("opendut-edgar-client".to_string()),
        ClientSecret::new(
            std::env::var("OPENDUT_EDGAR_NETWORK_OIDC_CLIENT_SECRET")
                .expect("OPENDUT_EDGAR_NETWORK_OIDC_CLIENT_SECRET environment variable not set in test environment")
        ),
        IssuerUrl::try_from("https://auth.opendut.local/realms/opendut/").unwrap(),
        vec![],
    ));
    let reqwest_client = localenv_reqwest_client.await;

    ConfidentialClient::from_client_config(client_config, reqwest_client)
        .expect("Could not create confidential client for EDGAR.")
}

#[fixture]
async fn confidential_netbird_client(#[future] localenv_reqwest_client: ReqwestClient) -> ConfidentialClientRef {
    opendut_util_core::testing::init_localenv_secrets();
    let reqwest_client = localenv_reqwest_client.await;
    let client_config = OidcClientConfig::Confidential(OidcConfidentialClientConfig::new(
        ClientId::new("netbird-backend".to_string()),
        ClientSecret::new(
            std::env::var("NETBIRD_MANAGEMENT_CLIENT_SECRET")
                .expect("NETBIRD_MANAGEMENT_CLIENT_SECRET environment variable not set in test environment")
        ),
        IssuerUrl::try_from("https://auth.opendut.local/realms/netbird/").unwrap(),
        vec![],
    ));

    ConfidentialClient::from_client_config(client_config, reqwest_client)
        .expect("Could not create confidential client for NetBird.")
}


#[test_with::env(OPENDUT_RUN_KEYCLOAK_INTEGRATION_TESTS)]
#[rstest]
#[tokio::test]
async fn test_confidential_client_get_token(#[future] confidential_edgar_client: ConfidentialClientRef) {
    /*
     * This test is ignored because it requires a running keycloak server from the test environment.
     * To run this test, execute the following command:
     * cargo test --package opendut-auth --all-features -- --include-ignored
     */
    let token = confidential_edgar_client.await.get_token().await.unwrap();
    assert!(token.value.len() > 100);
}

#[test_with::env(OPENDUT_RUN_KEYCLOAK_INTEGRATION_TESTS)]
#[rstest]
#[tokio::test]
async fn test_confidential_client_get_token2(#[future] confidential_netbird_client: ConfidentialClientRef) {
    /*
     * This test is ignored because it requires a running keycloak server from the test environment.
     * To run this test, execute the following command:
     * cargo test --package opendut-auth --all-features -- --include-ignored
     */
    let client = confidential_netbird_client.await;
    let token = client.get_token().await.expect("foo");
    println!("Token {}", token);
    let client = ConfidentialClient::build_client_with_middleware(client);
    let url = "https://netbird-api.opendut.local/api/users";

    #[allow(dead_code)]
    #[derive(Deserialize)]
    struct NetbirdUser {
        id: String,
        email: String,
        name: String,
    }
    let users = client
        .get(url)
        .send()
        .await
        .expect("Failed to request NetBird users")
        .json::<Vec<NetbirdUser>>()
        .await
        .expect("Failed to deserialize NetBird users");

    assert!(!users.is_empty());
}
