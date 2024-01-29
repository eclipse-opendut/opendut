#![cfg(test)]

///Verify compatibility with examples from here: https://docs.netbird.io/api

use std::result::Result;

use async_trait::async_trait;
use googletest::prelude::*;
use reqwest::Response;
use rstest::{fixture, rstest};
use serde_json::json;
use uuid::uuid;

use opendut_types::cluster::ClusterId;

use super::*;

#[rstest]
#[tokio::test]
async fn delete_peer(fixture: Fixture) -> anyhow::Result<()> {

    let requester = fixture.requester(|_, request| {
        assert_that!(request.method(), eq(&Method::DELETE));
        assert_that!(request.url().path(), eq("/api/peers/ah8cca16lmn67acg5s11"));
        assert_that!(request.body(), none());
        Ok(Response::from(http::Response::builder()
            .body("")
            .unwrap()))
    });

    let client = Client::create(fixture.base_url(), None, Some(Box::new(requester)))?;

    let result = client.delete_netbird_peer(&netbird::PeerId::from("ah8cca16lmn67acg5s11")).await;

    assert_that!(result, ok(anything()));

    Ok(())
}

#[rstest]
#[tokio::test]
async fn create_group(fixture: Fixture) -> anyhow::Result<()> {
    let requester = fixture.requester(|fixture, request| {
        let request = request.body().unwrap()
            .as_bytes().unwrap();
        let request: serde_json::Value = serde_json::from_slice(request).unwrap();

        let expectation = json!({
                "name": fixture.cluster_netbird_group_name(),
                "peers": [
                    fixture.netbird_peer_id(),
                ]
            });

        assert_that!(request, eq(expectation));

        let response = http::Response::builder()
            .body(
                json!({
                            "id": "ch8i4ug6lnn4g9hqv7m0",
                            "name": fixture.cluster_netbird_group_name(),
                            "peers_count": 1,
                            "issued": "api",
                            "peers": [
                                {
                                    "id": "chacbco6lnnbn6cg5s90",
                                    "name": "stage-host-1",
                                }
                            ]
                        }).to_string()
            ).unwrap();

        Ok(Response::from(response))
    });

    let client = Client::create(fixture.base_url(), None, Some(Box::new(requester)))?;

    let result = client.create_netbird_group(
        fixture.cluster_id().into(),
        vec![fixture.netbird_peer_id()],
    ).await?;

    assert_that!(
            result,
            matches_pattern!(
                netbird::Group {
                    id: anything(),
                    name: eq(fixture.cluster_netbird_group_name()),
                    peers_count: anything(),
                    peers: elements_are!(
                        matches_pattern!(
                            netbird::GroupPeerInfo {
                                id: eq(fixture.netbird_peer_id()),
                                name: anything()
                            }
                        )
                    ),
                }
            )
        );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn find_group(fixture: Fixture) -> anyhow::Result<()> {

    let requester = fixture.requester(|fixture, request| {

        assert_that!(request.body(), none());
        let response = http::Response::builder()
            .body(
                json!([
                        {
                            "id": "ch8i4ug6lnn4g9hqv7m0",
                            "name": String::from(fixture.peer_netbird_group_name()),
                            "peers_count": 0,
                            "issued": "api",
                            "peers": [
                            ]
                        }
                    ]).to_string()
            ).unwrap();

        Ok(Response::from(response))
    });

    let client = Client::create(fixture.base_url, None, Some(Box::new(requester)))?;

    let result = client.get_netbird_group(&fixture.peer_netbird_group_name).await;

    assert_that!(
            result,
            ok(
                matches_pattern!(
                    netbird::Group {
                        id: eq(netbird::GroupId::from("ch8i4ug6lnn4g9hqv7m0")),
                        name: eq(fixture.peer_netbird_group_name),
                        peers_count: eq(0),
                        peers: empty(),
                    }
                )
            )
        );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn delete_group(fixture: Fixture) -> anyhow::Result<()> {

    let requester = fixture.requester(|_, request| {
        assert_that!(request.method(), eq(&Method::DELETE));
        assert_that!(request.url().path(), eq("/api/groups/aax77acflma44h075aa3"));
        assert_that!(request.body(), none());
        Ok(Response::from(http::Response::builder()
            .body("")
            .unwrap()))
    }
    );

    let client = Client::create(fixture.base_url, None, Some(Box::new(requester)))?;

    let result = client.delete_netbird_group(&netbird::GroupId::from("aax77acflma44h075aa3")).await;

    assert_that!(result, ok(anything()));

    Ok(())
}

#[rstest]
#[tokio::test]
async fn create_a_setup_key(fixture: Fixture) -> anyhow::Result<()> {

    let requester = fixture.requester(|fixture, request| {
        match request.url().path() {
            "/api/groups" => {
                assert_that!(request.body(), none());

                let response = http::Response::builder()
                    .body(
                        json!([
                                {
                                    "id": fixture.netbird_group_id(),
                                    "name": netbird::GroupName::from(fixture.peer_id()),
                                    "peers_count": 0,
                                    "issued": "api",
                                    "peers": []
                                }
                            ]).to_string()
                    ).unwrap();

                Ok(Response::from(response))
            }
            "/api/setup-keys" => {
                let request = request.body().unwrap()
                    .as_bytes().unwrap();
                let request: serde_json::Value = serde_json::from_slice(request).unwrap();

                let expectation = json!({
                        "name": fixture.peer_setup_key_name(),
                        "type": "reusable",
                        "expires_in": 86400,
                        "revoked": false,
                        "auto_groups": [
                            fixture.netbird_group_id()
                        ],
                        "usage_limit": 0,
                    });

                assert_that!(request, eq(expectation));

                let response = http::Response::builder()
                    .body(
                        json!({
                                "id": "2531583362",
                                "key": "A616097E-FCF0-48FA-9354-CA4A61142761",
                                "name": fixture.peer_setup_key_name(),
                                "expires": "2023-06-01T14:47:22.291057Z",
                                "type": "reusable",
                                "valid": true,
                                "revoked": false,
                                "used_times": 2,
                                "last_used": "2023-05-05T09:00:35.477782Z",
                                "state": "valid",
                                "auto_groups": [
                                    fixture.netbird_group_id()
                                ],
                                "updated_at": "2023-05-05T09:00:35.477782Z",
                                "usage_limit": 0
                            }).to_string()
                    ).unwrap();

                Ok(Response::from(response))
            }
            value => {
                panic!("Unexpected url path {value}")
            }
        }
    });

    let client = Client::create(fixture.base_url(), None, Some(Box::new(requester)))?;

    let result = client.create_netbird_setup_key(fixture.peer_id()).await?;

    assert_that!(
            result,
            matches_pattern!(
                netbird::SetupKey {
                    id: anything(),
                    key: anything(),
                    name: eq(fixture.peer_setup_key_name()),
                    expires: anything(),
                    r#type: eq(netbird::SetupKeyType::Reusable),
                    valid: eq(true),
                    revoked: eq(false),
                    used_times: anything(),
                    last_used: anything(),
                    state: eq(netbird::SetupKeyState::Valid),
                    auto_groups: eq(vec![fixture.netbird_group_id.0]),
                    updated_at: anything(),
                    usage_limit: eq(0),
                }
            )
        );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn create_access_control_rule(fixture: Fixture) -> anyhow::Result<()> {

    let requester = fixture.requester(|fixture, request| {
        let request = request.body().unwrap()
            .as_bytes().unwrap();
        let request: serde_json::Value = serde_json::from_slice(request).unwrap();

        let expectation = json!({
            "name": String::from(fixture.netbird_cluster_rule_name()),
            "description": fixture.netbird_cluster_rule_name().description(),
            "disabled": false,
            "flow": "bidirect",
            "sources": [fixture.netbird_group_id()],
            "destinations": [fixture.netbird_group_id()],
        });

        assert_that!(request, eq(expectation));

        let group = json!({
            "id": fixture.netbird_group_id(),
            "name": String::from(fixture.netbird_cluster_rule_name()),
            "peers_count": 0,
            "issued": "api"
        });

        let response = http::Response::builder()
            .body(
                json!({
                    "id": "ch8i4ug6lnn4g9hqv7mg",
                    "name": String::from(fixture.netbird_cluster_rule_name()),
                    "description": fixture.netbird_cluster_rule_name().description(),
                    "disabled": false,
                    "flow": "bidirect",
                    "sources": [group],
                    "destinations": [group]
                }).to_string()
            ).unwrap();

        Ok(Response::from(response))
    });

    let client = Client::create(fixture.base_url(), None, Some(Box::new(requester)))?;

    let group = netbird::Group {
        id: fixture.netbird_group_id(),
        name: fixture.cluster_id().into(),
        peers_count: 0,
        peers: vec![],
    };

    client.create_netbird_self_access_control_rule(
        group,
        fixture.cluster_id().into(),
    ).await?;

    Ok(())
}

#[fixture]
fn fixture() -> Fixture {
    let base_url = Url::parse("https://localhost/api/").unwrap();
    let peer_id = PeerId::from(uuid!("b7dd1960-9ab5-4f3a-851d-6b68a90099eb"));
    let cluster_id = ClusterId::from(uuid!("999f8513-d7ab-43fe-9bf0-091abaff2a97"));
    let netbird_group_id = netbird::GroupId::from("ch8i4ug6lnn4g9hqv7m0");
    let cluster_netbird_group_name = netbird::GroupName::Cluster(cluster_id);
    let peer_netbird_group_name = netbird::GroupName::Peer(peer_id);
    let netbird_peer_id = netbird::PeerId(String::from("chacbco6lnnbn6cg5s90"));
    let netbird_peer_setup_key_name = netbird::setup_key_name_format(peer_id);
    let netbird_cluster_rule_name = netbird::RuleName::Cluster(cluster_id).into();
    Fixture {
        base_url,
        peer_id,
        cluster_id,
        netbird_group_id,
        cluster_netbird_group_name,
        peer_netbird_group_name,
        netbird_peer_id,
        netbird_peer_setup_key_name,
        netbird_cluster_rule_name
    }
}

#[derive(Clone)]
struct Fixture {
    base_url: Url,
    peer_id: PeerId,
    cluster_id: ClusterId,
    netbird_group_id: netbird::GroupId,
    cluster_netbird_group_name: netbird::GroupName,
    peer_netbird_group_name: netbird::GroupName,
    netbird_peer_id: netbird::PeerId,
    netbird_peer_setup_key_name: String,
    netbird_cluster_rule_name: netbird::RuleName,
}

impl Fixture {
    pub fn base_url(&self) -> Url {
        Clone::clone(&self.base_url)
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn cluster_id(&self) -> ClusterId {
        self.cluster_id
    }

    pub fn netbird_group_id(&self) -> netbird::GroupId {
        Clone::clone(&self.netbird_group_id)
    }

    pub fn cluster_netbird_group_name(&self) -> netbird::GroupName {
        Clone::clone(&self.cluster_netbird_group_name)
    }

    pub fn peer_netbird_group_name(&self) -> netbird::GroupName {
        Clone::clone(&self.peer_netbird_group_name)
    }

    pub fn netbird_peer_id(&self) -> netbird::PeerId {
        Clone::clone(&self.netbird_peer_id)
    }

    pub fn peer_setup_key_name(&self) -> String {
        Clone::clone(&self.netbird_peer_setup_key_name)
    }

    pub fn netbird_cluster_rule_name(&self) -> netbird::RuleName {
        Clone::clone(&self.netbird_cluster_rule_name)
    }

    pub fn requester<F>(&self, handler: F) -> MockRequester<F>
    where
        F: Fn(Fixture, Request) -> Result<Response, RequestError> + Send + Sync
    {
        MockRequester {
            fixture: Clone::clone(&self),
            handler,
        }
    }
}

struct MockRequester<F>
where
    F: Fn(Fixture, Request) -> Result<Response, RequestError> + Send + Sync
{
    fixture: Fixture,
    handler: F
}

#[async_trait]
impl <F> RequestHandler for MockRequester<F>
where
    F: Fn(Fixture, Request) -> Result<Response, RequestError> + Send + Sync
{
    async fn handle(&self, request: Request) -> Result<Response, RequestError> {
        (self.handler)(Clone::clone(&self.fixture), request)
    }
}
