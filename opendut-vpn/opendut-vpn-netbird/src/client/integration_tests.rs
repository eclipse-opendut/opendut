#![cfg(test)]

use std::time::Duration;

use reqwest::Url;
use rstest::rstest;
use uuid::uuid;

use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;
use opendut_util::project;
use opendut_vpn::VpnManagementClient;

use crate::{netbird, NetbirdManagementClient, NetbirdManagementClientConfiguration, NetbirdToken};
use crate::client::{Client, DefaultClient};

#[test_with::env(NETBIRD_INTEGRATION_API_TOKEN)]
#[rstest]
#[tokio::test]
async fn test_netbird_management_client() {
    /*
     * designated to be run in the opendut-vm, requires netbird management service to be running
     * API_KEY is fetched from the init container
     docker exec -ti netbird-management_init-1 cat /management/api_key
     export NETBIRD_INTEGRATION_API_TOKEN=<tbd>
     cargo test --package opendut-vpn-netbird test_foo --all-features -- --nocapture
     */
    let management_url = Url::parse("https://netbird-management/api/").unwrap();
    let netbird_api_token = std::env::var("NETBIRD_INTEGRATION_API_TOKEN").expect("Could not get netbird api token!");
    let authentication_token = NetbirdToken::new_personal_access(netbird_api_token);
    let timeout = Duration::from_millis(10000);
    let ca_path = project::make_path_absolute("resources/development/tls/insecure-development-ca.pem").expect("Could not determine ca path.");

    let netbird_management_client = NetbirdManagementClient::create(
        NetbirdManagementClientConfiguration {
            management_url: Clone::clone(&management_url),
            authentication_token: Some(authentication_token),
            ca: Some(ca_path),
            timeout,
            retries: 3,
            setup_key_expiration: Duration::from_millis(86400000),
        }
    ).expect("Netbird management client could not be created!");

    let peer_id = PeerId::random();
    netbird_management_client.create_peer(peer_id).await.expect("Could not create NetBird peer");
    netbird_management_client.delete_peer(peer_id).await.expect("Could not delete NetBird peer");

}

#[test_with::env(NETBIRD_INTEGRATION_API_TOKEN)]
#[rstest]
#[tokio::test]
async fn test_netbird_vpn_client() {
    /*
     * designated to be run in the opendut-vm, requires netbird management service to be running
     * API_KEY is fetched from the init container
     docker exec -ti netbird-management_init-1 cat /management/api_key
     export NETBIRD_INTEGRATION_API_TOKEN=<tbd>
     cargo test --package opendut-vpn-netbird test_foo --all-features -- --nocapture
     */
    let management_url = Url::parse("https://netbird-management/api/").unwrap();
    let netbird_api_token = std::env::var("NETBIRD_INTEGRATION_API_TOKEN").expect("Could not get netbird api token!");
    let authentication_token = NetbirdToken::new_personal_access(netbird_api_token);
    let timeout = Duration::from_millis(10000);

    let client = DefaultClient::create(
        Clone::clone(&management_url),
        None,
        Some(authentication_token.clone()),
        None,
        timeout,
        3,
        Duration::from_millis(86400000),
    ).expect("Should be able to create netbird client!");

    let peer_id = PeerId::random();
    let cluster_id = ClusterId::from(uuid!("999f8513-d7ab-43fe-9bf0-091abaff2a97"));
    let netbird_group_name = netbird::GroupName::Peer(peer_id);

    let netbird_group = client.create_netbird_group(Clone::clone(&netbird_group_name), Vec::new()).await
        .expect("Could not create group");

    let _setup_key = client.generate_netbird_setup_key(peer_id).await
        .expect("Could not generate setup key!");

    client.create_netbird_self_access_control_policy(netbird_group.clone(), cluster_id.into()).await
        .expect("Could not create self access control policy");
    println!("Netbird group {:?} contains peers: {:?}", netbird_group.id, netbird_group.peers);

    for peer_info in netbird_group.peers {
        let error_message = format!("Could not delete peer: {:?}", peer_info.id);
        println!("Deleting NetBird peer: {:?}", peer_info);
        client.delete_netbird_peer(&peer_info.id)
            .await
            .expect(error_message.as_str());
    }

}