use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::{Body, header, Method, Request, Url};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;

use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;
use opendut_types::vpn::VpnPeerConfig;
use opendut_vpn::{CreateClusterError, CreatePeerError, DeleteClusterError, DeletePeerError, GetOrCreateConfigurationError, VpnManagementClient};

use crate::{netbird, NetbirdToken, routes};
use crate::client::request_handler::{DefaultRequestHandler, RequestHandler};
use crate::netbird::{access_control, error};
use crate::netbird::error::{CreateSetupKeyError, RequestError, GetGroupError};

pub struct Client {
    base_url: Url,
    requester: Box<dyn RequestHandler + Send + Sync>,
}

#[async_trait]
impl VpnManagementClient for Client {

    async fn create_cluster(&self, cluster_id: ClusterId, peers: &Vec<PeerId>) -> Result<(), CreateClusterError> {

        match self.delete_cluster(cluster_id).await {
            Ok(_) => log::debug!("Deleted a previous cluster with ID <{cluster_id}> before creating the new cluster."),
            Err(cause) => match cause {
                DeleteClusterError::NotFound { cluster_id, message } => log::trace!("Did not need to delete a previous cluster with ID <{cluster_id}> before creating the new cluster. ({message})"),
                DeleteClusterError::DeletionFailure { cluster_id, error } => {
                    return Err(CreateClusterError::CreationFailure { cluster_id, error: anyhow!("Failure while deleting a previous cluster with ID <{cluster_id}> before creating the new cluster: {error}").into() });
                }
            }
        };

        let netbird_peers: Vec<netbird::PeerId> = {
            let mut netbird_peers = vec![];
            for peer_id in peers {
                let group = self.get_netbird_group(&(*peer_id).into()).await
                    .map_err(|error| CreateClusterError::PeerResolutionFailure { peer_id: *peer_id, cluster_id, error: error.into()})?;
                let peer = group.peers.into_iter().next()
                    .ok_or(CreateClusterError::PeerResolutionFailure { peer_id: *peer_id, cluster_id, error: anyhow!("Self-Group does not contain expected peer!").into() })?;
                netbird_peers.push(peer.id);
            }
            netbird_peers
        };

        let group = self.create_netbird_group(cluster_id.into(), netbird_peers).await
            .map_err(|error| CreateClusterError::CreationFailure { cluster_id, error: error.into() })?;

        self.create_netbird_self_access_control_rule(group, cluster_id).await
            .map_err(|error| CreateClusterError::AccessControlRuleCreationFailure { cluster_id, error: error.into() })?;

        Ok(())
    }

    async fn delete_cluster(&self, cluster_id: ClusterId) -> Result<(), DeleteClusterError> {
        let groups = self.get_all_netbird_groups(&netbird::GroupName::from(cluster_id)).await
            .map_err(|cause| DeleteClusterError::DeletionFailure { cluster_id, error: anyhow!("Failed to request the list of groups to determine which should be deleted:\n  {cause}").into() })?;

        if groups.is_empty() {
            return Err(DeleteClusterError::NotFound { cluster_id, message: format!("No group matching the cluster <{cluster_id}> found.") })
        } else {
            for group in groups {
                match self.delete_netbird_group(&group.id).await {
                    Ok(_) => log::debug!("Deleted NetBird group with name '{}' and NetBird Group ID '{}'.", group.name, group.id.0),
                    Err(cause) => return match cause {
                        RequestError::IllegalStatus(error) => {
                            if let Some(http::StatusCode::NOT_FOUND) = error.status() {
                                Err(DeleteClusterError::NotFound { cluster_id, message: format!("Received '404 Not Found' when deleting cluster <{cluster_id}> with NetBird group ID <{netbird_group}>.", netbird_group=group.id.0) })
                            } else {
                                Err(DeleteClusterError::DeletionFailure { cluster_id, error: error.into() }) //TODO logging of this doesn't show the HTTP body, making e.g. 400 Bad Request errors difficult to debug
                            }
                        },
                        other => Err(DeleteClusterError::DeletionFailure { cluster_id, error: other.into() }),
                    }
                }
            }
        }

        //TODO delete access control rule in NetBird

        Ok(())
    }

    async fn create_peer(&self, peer_id: PeerId) -> Result<(), CreatePeerError> {
        let peers = vec![]; //Peer self-group does not have peers
        self.create_netbird_group(peer_id.into(), peers).await
            .map_err(|error| CreatePeerError::CreationFailure { peer_id, error: error.into() })?;
        Ok(())
    }

    async fn delete_peer(&self, peer_id: PeerId) -> Result<(), DeletePeerError> {

        let self_group = self.get_netbird_group(&netbird::GroupName::from(peer_id)).await
            .map_err(|error| DeletePeerError::ResolutionFailure { peer_id, error: error.into() })?;

        if let Some(peer_info) = self_group.peers.get(0) {
            self.delete_netbird_peer(&peer_info.id)
                .await
                .map_err(|error| DeletePeerError::DeletionFailure { peer_id, error: error.into() })?;
        }

        self.delete_netbird_group(&self_group.id)
            .await
            .map_err(|error| DeletePeerError::DeletionFailure { peer_id, error: error.into() })?;

        Ok(())
    }

    async fn get_or_create_configuration(&self, peer_id: PeerId) -> Result<VpnPeerConfig, GetOrCreateConfigurationError> {

        let setup_keys = self.list_netbird_setup_keys().await
            .map_err(|error| GetOrCreateConfigurationError::QueryConfigurationsFailure { error: error.into() })?;

        let maybe_setup_key = setup_keys.into_iter()
            .find(|setup_key| setup_key.name == netbird::setup_key::name_format(peer_id));

        let setup_key = match maybe_setup_key {
            None => {
                self.create_netbird_setup_key(peer_id).await
                    .map_err(|error| GetOrCreateConfigurationError::CreationFailure { peer_id, error: error.into() })?
            }
            Some(setup_key) => setup_key,
        };

        Ok(VpnPeerConfig::Netbird {
            management_url: self.base_url.clone(),
            setup_key: opendut_types::vpn::netbird::SetupKey::from(setup_key.key),
        })
    }
}

impl Client {

    pub fn create(base_url: Url, token: NetbirdToken) -> Result<Self, error::CreateClientError> {
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
            .https_only(true)
            .build()
            .expect("Failed to construct client.");

        let requester = Box::new(DefaultRequestHandler::from(client));
        Ok(Self {
            base_url,
            requester,
        })
    }

    async fn create_netbird_group(&self, name: netbird::GroupName, peers: Vec<netbird::PeerId>) -> Result<netbird::Group, RequestError> {

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

    async fn get_all_netbird_groups(&self, group_name: &netbird::GroupName) -> Result<Vec<netbird::Group>, GetGroupError> {
        let url = routes::groups(self.base_url.clone());
        let request = Request::new(Method::GET, url);

        let response = self.requester.handle(request).await
            .map_err(|cause| GetGroupError::RequestFailure { group_name: group_name.to_owned(), cause })?;

        let result = response.json::<Vec<netbird::Group>>().await
            .map_err(|cause| GetGroupError::RequestFailure { group_name: group_name.to_owned(), cause: RequestError::JsonDeserialization(cause) })?;

        let result = result.into_iter()
            .filter(|group| group.name == *group_name)
            .collect::<Vec<_>>();
        Ok(result)
    }

    async fn get_netbird_group(&self, group_name: &netbird::GroupName) -> Result<netbird::Group, GetGroupError> { //TODO remove? Introduce error for multiple? Rename to 'find' and 'filter'?
        let url = routes::groups(self.base_url.clone());
        let request = Request::new(Method::GET, url);

        let response = self.requester.handle(request).await
            .map_err(|cause| GetGroupError::RequestFailure { group_name: group_name.to_owned(), cause })?;

        let result = response.json::<Vec<netbird::Group>>().await
            .map_err(|cause| GetGroupError::RequestFailure { group_name: group_name.to_owned(), cause: RequestError::JsonDeserialization(cause) })?;

        result.into_iter()
            .find(|group| group.name == *group_name)
            .ok_or(GetGroupError::GroupNotFound { group_name: group_name.to_owned() })
    }

    async fn delete_netbird_group(&self, group_id: &netbird::GroupId) -> Result<(), RequestError> {

        let url = routes::group(Clone::clone(&self.base_url), &group_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        response.error_for_status()
            .map_err(RequestError::IllegalStatus)?;

        Ok(())
    }

    async fn create_netbird_self_access_control_rule(&self, group: netbird::Group, cluster_id: ClusterId) -> Result<(), RequestError> {
        let url = routes::rules(self.base_url.clone());

        let body = {
            #[derive(Serialize)]
            struct CreateAccessControlRule {
                name: String,
                description: String,
                disabled: bool,
                flow: access_control::RuleFlow,
                sources: Vec<netbird::GroupId>,
                destinations: Vec<netbird::GroupId>,
            }

            CreateAccessControlRule {
                name: access_control::rule_name(cluster_id),
                description: access_control::rule_description(cluster_id),
                disabled: false,
                flow: access_control::RuleFlow::Bidirect,
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

    async fn delete_netbird_peer(&self, peer_id: &netbird::PeerId) -> Result<(), RequestError> {

        let url = routes::peer(Clone::clone(&self.base_url), peer_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        response.error_for_status()
            .map_err(RequestError::IllegalStatus)?;

        Ok(())
    }

    async fn create_netbird_setup_key(&self, peer_id: PeerId) -> Result<netbird::SetupKey, CreateSetupKeyError> {

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
                r#type: netbird::setup_key::Type::OneOff,
                expires_in: 24*60*60, //TODO make configurable
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

    async fn list_netbird_setup_keys(&self) -> Result<Vec<netbird::SetupKey>, RequestError> {
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


///Verify compatibility with examples from here: https://docs.netbird.io/api
#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use googletest::prelude::*;
    use std::result::Result;
    use reqwest::Response;
    use serde_json::json;
    use uuid::uuid;

    use opendut_types::cluster::ClusterId;

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
    async fn delete_a_peer() -> anyhow::Result<()> {

        fn peer_id() -> PeerId { PeerId::from(uuid!("f3fb5772-a259-400e-9e61-b5a69dc46c2a")) }

        let requester = MockRequester {
            on_request: |request| {
                match request.url().path() {
                    "/api/groups" => {
                        assert_that!(request.body(), none());

                        let response = http::Response::builder()
                            .body(json!(
                                [
                                    {
                                        "id": "aax77acflma44h075aa3",
                                        "name": netbird::GroupName::from(peer_id()),
                                        "peers_count": 1,
                                        "issued": "api",
                                        "peers": [
                                            {
                                                "id": "ah8cca16lmn67acg5s11",
                                                "name": "some-peer"
                                            }
                                        ]
                                    }
                                ]
                            ).to_string()).unwrap();

                        Ok(Response::from(response))
                    }
                    path => {
                        if path.starts_with("/api/groups/") {
                            assert_that!(request.method(), eq(&http::Method::DELETE));
                            assert_that!(request.url().path(), eq("/api/groups/aax77acflma44h075aa3"));
                            assert_that!(request.body(), none());
                            Ok(Response::from(http::Response::builder()
                                .body("")
                                .unwrap()))
                        }
                        else if path.starts_with("/api/peers/") {
                            assert_that!(request.method(), eq(&http::Method::DELETE));
                            assert_that!(request.url().path(), eq("/api/peers/ah8cca16lmn67acg5s11"));
                            assert_that!(request.body(), none());
                            Ok(Response::from(http::Response::builder()
                                .body("")
                                .unwrap()))
                        }
                        else {
                            panic!("Unexpected url path: {path}");
                        }
                    }
                }
            },
        };

        let client = Client {
            base_url: base_url(),
            requester: Box::new(requester),
        };

        let result = client.delete_peer(peer_id()).await;

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
                        id: eq(netbird::GroupId::from("ch8i4ug6lnn4g9hqv7m0")),
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
    async fn create_a_setup_key() -> anyhow::Result<()> {

        fn peer_id() -> PeerId { PeerId::from(uuid!("b7dd1960-9ab5-4f3a-851d-6b68a90099eb")) }
        fn name() -> String { netbird::setup_key::name_format(peer_id()) }
        fn netbird_group_id() -> netbird::GroupId { netbird::GroupId::from("ch8i4ug6lnn4g9hqv7m0") }

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
                            "type": "one-off",
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
                                    "type": "one-off",
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
                    r#type: eq(netbird::setup_key::Type::OneOff),
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
        fn name() -> String { access_control::rule_name(cluster_id()) }
        fn description() -> String {
            access_control::rule_description(cluster_id())
        }
        fn netbird_group_id() -> netbird::GroupId {
            netbird::GroupId::from("ch8i4ug6lnn4g9hqv7m0")
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
            cluster_id(),
        ).await?;

        Ok(())
    }
}
