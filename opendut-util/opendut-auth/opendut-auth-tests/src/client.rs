use oauth2::{ClientId, ClientSecret};
use rstest::{fixture, rstest};
use url::Url;
use opendut_auth::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use opendut_auth::confidential::config::ConfidentialClientConfigData;
use opendut_auth::confidential::reqwest_client::OidcReqwestClient;
use crate::localenv_reqwest_client;

#[fixture]
async fn confidential_edgar_client(#[future] localenv_reqwest_client: OidcReqwestClient) -> ConfidentialClientRef {
    opendut_util_core::testing::init_localenv_secrets();
    let client_config = ConfidentialClientConfigData::new(
        ClientId::new("opendut-edgar-client".to_string()),
        ClientSecret::new(
            std::env::var("OPENDUT_EDGAR_NETWORK_OIDC_CLIENT_SECRET")
                .expect("OPENDUT_EDGAR_NETWORK_OIDC_CLIENT_SECRET environment variable not set in test environment")
        ),
        Url::parse("https://auth.opendut.local/realms/opendut/").unwrap(),
        vec![],
    );
    let reqwest_client = localenv_reqwest_client.await;

    ConfidentialClient::from_client_config(client_config, reqwest_client).await.unwrap()
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