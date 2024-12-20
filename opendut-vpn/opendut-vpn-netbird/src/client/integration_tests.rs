#![cfg(test)]

use std::path::PathBuf;
use std::time::Duration;

use reqwest::Url;
use uuid::uuid;

use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;
use opendut_util::project;
use opendut_vpn::VpnManagementClient;

use crate::{netbird, NetbirdManagementClient, NetbirdManagementClientConfiguration, NetbirdToken};
use crate::client::{Client, DefaultClient};

#[test_with::env(NETBIRD_INTEGRATION_API_TOKEN)]
#[test_log::test(tokio::test)]
async fn test_netbird_management_client() {
    /*
     * Designated to be run in the opendut-vm, requires netbird management service to be running.
     * API_KEY is fetched from the init container.
     docker exec -ti netbird-management_init-1 cat /management/api_key
     export NETBIRD_INTEGRATION_API_TOKEN=<tbd>
     cargo test --package opendut-vpn-netbird test_foo --all-features -- --nocapture
     */
    let Fixture { management_url, authentication_token, ca, timeout, retries, setup_key_expiration } = Fixture::default();

    let netbird_management_client = NetbirdManagementClient::create_client_and_delete_default_policy(
        NetbirdManagementClientConfiguration {
            management_url: management_url,
            authentication_token: Some(authentication_token),
            ca: Some(ca),
            timeout,
            retries,
            setup_key_expiration,
        }
    ).await.expect("Netbird management client could not be created!");

    let peer_id = PeerId::random();
    netbird_management_client.create_peer(peer_id).await.expect("Could not create NetBird peer");
    netbird_management_client.delete_peer(peer_id).await.expect("Could not delete NetBird peer");
}

#[test_with::env(NETBIRD_INTEGRATION_API_TOKEN)]
#[tokio::test]
async fn test_netbird_vpn_client() {
    /*
     * Designated to be run in the opendut-vm, requires netbird management service to be running.
     * API_KEY is fetched from the init container.
     docker exec -ti netbird-management_init-1 cat /management/api_key
     export NETBIRD_INTEGRATION_API_TOKEN=<tbd>
     cargo test --package opendut-vpn-netbird test_foo --all-features -- --nocapture
     */
    let Fixture { management_url, authentication_token, ca: _, timeout, retries, setup_key_expiration } = Fixture::default();

    let client = DefaultClient::create(
        Clone::clone(&management_url),
        None,
        Some(authentication_token.clone()),
        None,
        timeout,
        retries,
        setup_key_expiration,
    ).expect("Should be able to create netbird client!");

    let peer_id = PeerId::random();
    let cluster_id = ClusterId::from(uuid!("999f8513-d7ab-43fe-9bf0-091abaff2a97"));
    let netbird_group_name = netbird::GroupName::Peer(peer_id);

    let netbird_group = client.create_netbird_group(Clone::clone(&netbird_group_name), Vec::new()).await
        .expect("Could not create group");

    let _setup_key = client.generate_netbird_setup_key(peer_id).await
        .expect("Could not generate setup key!");

    client.create_netbird_self_policy(netbird_group.clone(), cluster_id.into()).await
        .expect("Could not create self access control policy");
    println!("Netbird group {:?} contains peers: {:?}", netbird_group.id, netbird_group.peers);

    for peer_info in netbird_group.peers {
        let error_message = format!("Could not delete peer: {:?}", peer_info.id);
        println!("Deleting NetBird peer: {:?}", peer_info);
        client.delete_netbird_peer(&peer_info.id)
            .await
            .expect(&error_message);
    }
}

struct Fixture {
    pub management_url: Url,
    pub authentication_token: NetbirdToken,
    pub ca: PathBuf,
    pub timeout: Duration,
    pub retries: u32,
    pub setup_key_expiration: Duration,
}
impl Default for Fixture {
    fn default() -> Self {
        let management_url = Url::parse("https://netbird-management/api/").unwrap();
        let netbird_api_token = std::env::var("NETBIRD_INTEGRATION_API_TOKEN").expect("Could not get netbird api token!");
        let authentication_token = NetbirdToken::new_personal_access(netbird_api_token);
        let timeout = Duration::from_millis(10000);
        let ca = project::make_path_absolute("resources/development/tls/insecure-development-ca.pem").expect("Could not determine ca path.");

        Self {
            management_url,
            authentication_token,
            ca,
            timeout,
            retries: 3,
            setup_key_expiration: Duration::from_millis(86400000),
        }
    }
}
