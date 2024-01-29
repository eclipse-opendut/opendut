use std::time::Duration;

use http::{header, HeaderMap, HeaderValue, Method};
use reqwest::{Body, Request, Response, Url};
use serde::Serialize;

use opendut_types::peer::PeerId;
use opendut_types::vpn::HttpsOnly;

use crate::{netbird, NetbirdToken, routes};
use crate::client::request_handler::{DefaultRequestHandler, RequestHandler, RequestHandlerConfig};
use crate::netbird::{error, group, rules};
use crate::netbird::error::{CreateSetupKeyError, GetGroupError, GetRulesError, RequestError};
use crate::netbird::rules::RuleName;

mod request_handler;

const SETUP_KEY_EXPIRY_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

pub struct Client {
    base_url: Url,
    requester: Box<dyn RequestHandler + Send + Sync>,
}

impl Client {

    pub fn create(base_url: Url, token: NetbirdToken, https_only: HttpsOnly) -> Result<Self, error::CreateClientError> {
        let headers = {
            let mut headers = HeaderMap::new();
            headers.append(header::ACCEPT, json_header_value());

            let auth_header = token.sensitive_header()
                .map_err(error::CreateClientError::InvalidHeader)?;
            headers.append(header::AUTHORIZATION, auth_header);

            headers
        };

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(https_only.to_bool())
            .build()
            .expect("Failed to construct client.");

        let requester = Box::new(DefaultRequestHandler::new(
            client,
            RequestHandlerConfig::default(), //TODO pass in from the outside
        ));
        Ok(Self {
            base_url,
            requester,
        })
    }

    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    pub async fn create_netbird_group(&self, name: netbird::GroupName, peers: Vec<netbird::PeerId>) -> Result<netbird::Group, RequestError> {

        let url = routes::groups(self.base_url.clone());

        let body = {
            #[derive(Serialize)]
            struct CreateGroup {
                name: netbird::GroupName,
                peers: Vec<netbird::PeerId>,
            }

            CreateGroup {
                name,
                peers,
            }
        };

        let request = post_json_request(url, body)?;

        let response = self.requester.handle(request).await?
            .error_for_status().map_err(RequestError::IllegalStatus)?;

        let result = response.json().await
            .map_err(RequestError::JsonDeserialization)?;

        Ok(result)
    }

    pub async fn get_netbird_rule(&self, rule_name: &RuleName) -> Result<rules::Rule, GetRulesError> {
        let url = routes::rules(self.base_url.clone());
        let request = Request::new(Method::GET, url);
        let response = self.requester.handle(request).await
            .map_err(|cause| GetRulesError::RequestFailure { rule_name: rule_name.to_owned(), cause })?;
        let result = response.json::<Vec<rules::Rule>>().await
            .map_err(|cause| GetRulesError::RequestFailure { rule_name: rule_name.to_owned(), cause: RequestError::JsonDeserialization(cause) })?;

        let rules = result.into_iter()
            .filter(|rule| rule.name == *rule_name)
            .collect::<Vec<_>>();

        if rules.len() > 1 {
            Err(GetRulesError::MultipleRulesFound { rule_name: rule_name.to_owned() })
        } else {
            rules.into_iter().next().ok_or(GetRulesError::RuleNotFound { rule_name: rule_name.to_owned() })
        }
    }

    pub async fn get_netbird_group(&self, group_name: &netbird::GroupName) -> Result<netbird::Group, GetGroupError> {
        let url = routes::groups(self.base_url.clone());
        let request = Request::new(Method::GET, url);

        let response = self.requester.handle(request).await
            .map_err(|cause| GetGroupError::RequestFailure { group_name: group_name.to_owned(), cause })?;

        let result = response.json::<Vec<netbird::Group>>().await
            .map_err(|cause| GetGroupError::RequestFailure { group_name: group_name.to_owned(), cause: RequestError::JsonDeserialization(cause) })?;

        let groups = result.into_iter()
            .filter(|group| group.name == *group_name)
            .collect::<Vec<_>>();

        if groups.len() > 1 {
            Err(GetGroupError::MultipleGroupsFound { group_name: group_name.to_owned() })
        } else {
            groups.into_iter().next().ok_or(GetGroupError::GroupNotFound { group_name: group_name.to_owned() })
        }
    }

    pub async fn delete_netbird_group(&self, group_id: &group::GroupId) -> Result<(), RequestError> {
        let url = routes::group(Clone::clone(&self.base_url), group_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        parse_response_status(response, format!("NetBird group with ID <{:?}>", group_id)).await
    }

    pub async fn delete_netbird_rule(&self, rule_id: &rules::RuleId) -> Result<(), RequestError> {
        let url = routes::rule(Clone::clone(&self.base_url), rule_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        parse_response_status(response, format!("NetBird rule with ID <{:?}>", rule_id)).await
    }

    pub async fn create_netbird_self_access_control_rule(&self, group: netbird::Group, rule_name: RuleName) -> Result<(), RequestError> {
        let url = routes::rules(self.base_url.clone());

        let body = {
            #[derive(Serialize)]
            struct CreateAccessControlRule {
                name: RuleName,
                description: String,
                disabled: bool,
                flow: rules::RuleFlow,
                sources: Vec<group::GroupId>,
                destinations: Vec<group::GroupId>,
            }

            let description = rule_name.description();
            CreateAccessControlRule {
                name: rule_name,
                description,
                disabled: false,
                flow: rules::RuleFlow::Bidirect,
                sources: vec![group.id.clone()],
                destinations: vec![group.id],
            }
        };

        let request = post_json_request(url, body)?;

        let response = self.requester.handle(request).await?;
        response.error_for_status()
            .map_err(RequestError::IllegalStatus)?;

        Ok(())
    }

    pub async fn get_netbird_peer(&self, peer_id: &netbird::PeerId) -> Result<netbird::Peer, RequestError> {
        let url = routes::peer(Clone::clone(&self.base_url), peer_id);

        let request = Request::new(Method::GET, url);

        let response = self.requester.handle(request).await?
            .error_for_status().map_err(RequestError::IllegalStatus)?;

        let result = response.json::<netbird::Peer>().await
            .map_err(RequestError::JsonDeserialization)?;

        Ok(result)
    }

    pub async fn delete_netbird_peer(&self, peer_id: &netbird::PeerId) -> Result<(), RequestError> {
        let url = routes::peer(Clone::clone(&self.base_url), peer_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        response.error_for_status()
            .map_err(RequestError::IllegalStatus)?;

        Ok(())
    }

    pub async fn create_netbird_setup_key(&self, peer_id: PeerId) -> Result<netbird::SetupKey, CreateSetupKeyError> {
        let peer_group_name = netbird::GroupName::from(peer_id);
        let peer_group = self.get_netbird_group(&peer_group_name).await
            .map_err(|cause| CreateSetupKeyError::PeerGroupNotFound { peer_id, cause })?;

        let url = routes::setup_keys(self.base_url.clone());

        let body = {
            #[derive(Serialize)]
            struct CreateSetupKey {
                name: String,
                r#type: netbird::setup_key::Type,
                expires_in: u64, //seconds
                revoked: bool,
                auto_groups: Vec<String>,
                usage_limit: u64,
            }

            CreateSetupKey {
                name: netbird::setup_key::name_format(peer_id),
                r#type: netbird::setup_key::Type::Reusable,
                expires_in: SETUP_KEY_EXPIRY_DURATION.as_secs(),
                revoked: false,
                auto_groups: vec![
                    peer_group.id.0
                ],
                usage_limit: 0,
            }
        };

        let request = post_json_request(url, body)
            .map_err(|cause| CreateSetupKeyError::RequestFailure { peer_id, cause })?;

        let response = self.requester.handle(request).await
            .map_err(|cause| CreateSetupKeyError::RequestFailure { peer_id, cause })?
            .error_for_status().map_err(|cause| CreateSetupKeyError::RequestFailure { peer_id, cause: RequestError::IllegalStatus(cause) })?;

        let result = response.json().await
            .map_err(|cause| CreateSetupKeyError::RequestFailure { peer_id, cause: RequestError::JsonDeserialization(cause) })?;

        Ok(result)
    }

    pub async fn list_netbird_setup_keys(&self) -> Result<Vec<netbird::SetupKey>, RequestError> {
        let url = routes::setup_keys(self.base_url.clone());
        let request = Request::new(Method::GET, url);

        let response = self.requester.handle(request).await?;
        let result = response.json::<Vec<netbird::SetupKey>>().await
            .map_err(RequestError::JsonDeserialization)?;

        Ok(result)
    }
}

fn post_json_request(url: Url, body: impl Serialize) -> Result<Request, RequestError> {
    let mut request = Request::new(Method::POST, url);

    request.headers_mut()
        .insert(header::CONTENT_TYPE, json_header_value());

    let body = serde_json::to_vec(&body)
        .map_err(RequestError::JsonSerialization)?;

    *request.body_mut() = Some(Body::from(body));

    Ok(request)
}

fn json_header_value() -> HeaderValue {
    HeaderValue::from_str(mime::APPLICATION_JSON.as_ref())
        .expect("MIME type for application/json should always be convertable to a header value.")
}

async fn parse_response_status(response: Response, error_text: String) -> Result<(), RequestError> {
    match response.error_for_status_ref() {
        Ok(_) => Ok(()),
        Err(status) => {
            let body = response.text().await.unwrap_or(String::from("<no body>"));
            let status_code = status.status().expect("Error should be generated from a response");
            log::error!("Received status code '{code}' when deleting {error_text}:\n  {body}", code=status_code, error_text=error_text, body=body);
            Err(RequestError::IllegalRequest(status_code, body))
        }
    }
}

///Verify compatibility with examples from here: https://docs.netbird.io/api
#[cfg(test)]
mod tests {
    use std::result::Result;

    use async_trait::async_trait;
    use googletest::prelude::*;
    use reqwest::Response;
    use serde_json::json;
    use uuid::uuid;

    use opendut_types::cluster::ClusterId;

    use crate::netbird::group;
    use crate::netbird::group::GroupId;

    use super::*;

    fn base_url() -> Url {
        Url::parse("https://localhost/api/").unwrap()
    }

    struct MockRequester {
        on_request: fn(Request) -> Result<Response, RequestError>,
    }

    #[async_trait]
    impl RequestHandler for MockRequester {
        async fn handle(&self, request: Request) -> Result<Response, RequestError> {
            (self.on_request)(request)
        }
    }

    #[tokio::test]
    async fn delete_peer() -> anyhow::Result<()> {

        let requester = MockRequester {
            on_request: |request| {
                assert_that!(request.method(), eq(&Method::DELETE));
                assert_that!(request.url().path(), eq("/api/peers/ah8cca16lmn67acg5s11"));
                assert_that!(request.body(), none());
                Ok(Response::from(http::Response::builder()
                    .body("")
                    .unwrap()))
            },
        };

        let client = Client {
            base_url: base_url(),
            requester: Box::new(requester),
        };

        let result = client.delete_netbird_peer(&netbird::PeerId::from("ah8cca16lmn67acg5s11")).await;

        assert_that!(result, ok(anything()));

        Ok(())
    }

    #[tokio::test]
    async fn create_group() -> anyhow::Result<()> {
        fn cluster_id() -> ClusterId { ClusterId::from(uuid!("999f8513-d7ab-43fe-9bf0-091abaff2a97")) }
        fn name() -> netbird::GroupName { netbird::GroupName::Cluster(cluster_id()) }
        fn netbird_peer() -> netbird::PeerId { netbird::PeerId(String::from("chacbco6lnnbn6cg5s90")) }

        let requester = MockRequester {
            on_request: |request| {
                let request = request.body().unwrap()
                    .as_bytes().unwrap();
                let request: serde_json::Value = serde_json::from_slice(request).unwrap();

                let expectation = json!({
                    "name": name(),
                    "peers": [
                        netbird_peer(),
                    ]
                });

                assert_that!(request, eq(expectation));

                let response = http::Response::builder()
                    .body(
                        json!({
                            "id": "ch8i4ug6lnn4g9hqv7m0",
                            "name": name(),
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
            },
        };

        let client = Client {
            base_url: base_url(),
            requester: Box::new(requester),
        };

        let result = client.create_netbird_group(
            cluster_id().into(),
            vec![netbird_peer()],
        ).await?;

        assert_that!(
            result,
            matches_pattern!(
                netbird::Group {
                    id: anything(),
                    name: eq(name()),
                    peers_count: anything(),
                    peers: elements_are!(
                        matches_pattern!(
                            netbird::group::GroupPeerInfo {
                                id: eq(netbird_peer()),
                                name: anything()
                            }
                        )
                    ),
                }
            )
        );
        Ok(())
    }

    #[tokio::test]
    async fn find_group() -> anyhow::Result<()> {
        fn peer_id() -> PeerId { PeerId::from(uuid!("53b49ffb-9962-487a-b522-981ebe6aac59")) }
        fn name() -> netbird::GroupName { netbird::GroupName::Peer(peer_id()) }

        let requester = MockRequester {
            on_request: |request| {
                assert_that!(request.body(), none());
                let response = http::Response::builder()
                    .body(
                        json!([
                            {
                                "id": "ch8i4ug6lnn4g9hqv7m0",
                                "name": String::from(name()),
                                "peers_count": 0,
                                "issued": "api",
                                "peers": [
                                ]
                            }
                        ]).to_string()
                    ).unwrap();

                Ok(Response::from(response))
            },
        };

        let client = Client {
            base_url: base_url(),
            requester: Box::new(requester),
        };

        let result = client.get_netbird_group(&name()).await;

        assert_that!(
            result,
            ok(
                matches_pattern!(
                    netbird::Group {
                        id: eq(netbird::group::GroupId::from("ch8i4ug6lnn4g9hqv7m0")),
                        name: eq(name()),
                        peers_count: eq(0),
                        peers: empty(),
                    }
                )
            )
        );
        Ok(())
    }

    #[tokio::test]
    async fn delete_group() -> anyhow::Result<()> {

        let requester = MockRequester {
            on_request: |request| {
                assert_that!(request.method(), eq(&Method::DELETE));
                assert_that!(request.url().path(), eq("/api/groups/aax77acflma44h075aa3"));
                assert_that!(request.body(), none());
                Ok(Response::from(http::Response::builder()
                    .body("")
                    .unwrap()))
            }
        };

        let client = Client {
            base_url: base_url(),
            requester: Box::new(requester),
        };

        let result = client.delete_netbird_group(&GroupId::from("aax77acflma44h075aa3")).await;

        assert_that!(result, ok(anything()));

        Ok(())
    }

    #[tokio::test]
    async fn create_a_setup_key() -> anyhow::Result<()> {
        fn peer_id() -> PeerId { PeerId::from(uuid!("b7dd1960-9ab5-4f3a-851d-6b68a90099eb")) }
        fn name() -> String { netbird::setup_key::name_format(peer_id()) }
        fn netbird_group_id() -> group::GroupId { group::GroupId::from("ch8i4ug6lnn4g9hqv7m0") }

        let requester = MockRequester {
            on_request: |request| {
                match request.url().path() {
                    "/api/groups" => {
                        assert_that!(request.body(), none());

                        let response = http::Response::builder()
                            .body(
                                json!([
                                    {
                                        "id": netbird_group_id(),
                                        "name": netbird::GroupName::from(peer_id()),
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
                            "name": name(),
                            "type": "reusable",
                            "expires_in": 86400,
                            "revoked": false,
                            "auto_groups": [
                                netbird_group_id()
                            ],
                            "usage_limit": 0,
                        });

                        assert_that!(request, eq(expectation));

                        let response = http::Response::builder()
                            .body(
                                json!({
                                    "id": "2531583362",
                                    "key": "A616097E-FCF0-48FA-9354-CA4A61142761",
                                    "name": name(),
                                    "expires": "2023-06-01T14:47:22.291057Z",
                                    "type": "reusable",
                                    "valid": true,
                                    "revoked": false,
                                    "used_times": 2,
                                    "last_used": "2023-05-05T09:00:35.477782Z",
                                    "state": "valid",
                                    "auto_groups": [
                                        netbird_group_id()
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
            },
        };

        let client = Client {
            base_url: base_url(),
            requester: Box::new(requester),
        };

        let result = client.create_netbird_setup_key(peer_id()).await?;

        assert_that!(
            result,
            matches_pattern!(
                netbird::SetupKey {
                    id: anything(),
                    key: anything(),
                    name: eq(name()),
                    expires: anything(),
                    r#type: eq(netbird::setup_key::Type::Reusable),
                    valid: eq(true),
                    revoked: eq(false),
                    used_times: anything(),
                    last_used: anything(),
                    state: eq(netbird::setup_key::State::Valid),
                    auto_groups: eq(vec![netbird_group_id().0]),
                    updated_at: anything(),
                    usage_limit: eq(0),
                }
            )
        );
        Ok(())
    }

    #[tokio::test]
    async fn create_access_control_rule() -> anyhow::Result<()> {
        fn cluster_id() -> ClusterId { ClusterId::from(uuid!("999f8513-d7ab-43fe-9bf0-091abaff2a97")) }
        fn name() -> String { RuleName::Cluster(cluster_id()).into() }
        fn description() -> String {
            RuleName::Cluster(cluster_id()).description()
        }

        fn netbird_group_id() -> group::GroupId {
            group::GroupId::from("ch8i4ug6lnn4g9hqv7m0")
        }

        let requester = MockRequester {
            on_request: |request| {
                let request = request.body().unwrap()
                    .as_bytes().unwrap();
                let request: serde_json::Value = serde_json::from_slice(request).unwrap();

                let expectation = json!({
                    "name": name(),
                    "description": description(),
                    "disabled": false,
                    "flow": "bidirect",
                    "sources": [netbird_group_id()],
                    "destinations": [netbird_group_id()],
                });

                assert_that!(request, eq(expectation));

                let group = json!({
                    "id": netbird_group_id(),
                    "name": name(),
                    "peers_count": 0,
                    "issued": "api"
                });

                let response = http::Response::builder()
                    .body(
                        json!({
                            "id": "ch8i4ug6lnn4g9hqv7mg",
                            "name": name(),
                            "description": description(),
                            "disabled": false,
                            "flow": "bidirect",
                            "sources": [group],
                            "destinations": [group]
                        }).to_string()
                    ).unwrap();

                Ok(Response::from(response))
            },
        };

        let client = Client {
            base_url: base_url(),
            requester: Box::new(requester),
        };

        let group = netbird::Group {
            id: netbird_group_id(),
            name: cluster_id().into(),
            peers_count: 0,
            peers: vec![],
        };
        client.create_netbird_self_access_control_rule(
            group,
            cluster_id().into(),
        ).await?;

        Ok(())
    }
}
