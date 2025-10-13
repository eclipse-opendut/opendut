use std::time::Duration;

use async_trait::async_trait;
use http::{header, HeaderMap, Method};
use reqwest::{Body, Certificate, Request, Response, Url};
use serde::Serialize;
use tracing::error;
use opendut_auth::confidential::client::ConfidentialClient;
use opendut_auth::confidential::{ClientId, ClientSecret};
use opendut_auth::confidential::config::OidcConfidentialClientConfig;
use opendut_auth::confidential::reqwest_client::OidcReqwestClient;
use opendut_model::peer::PeerId;

use crate::{netbird, routes};
use crate::client::auth::create_api_token;
use crate::client::request_handler::{DefaultRequestHandler, RequestHandler, RequestHandlerConfig};
use crate::netbird::error::{CreateClientError, CreateSetupKeyError, GetGroupError, GetPoliciesError, RequestError};

mod request_handler;

mod tests;
mod integration_tests;
mod auth;

#[async_trait]
pub trait Client {
    async fn create_netbird_group(&self, name: netbird::GroupName, peers: Vec<netbird::PeerId>) -> Result<netbird::Group, RequestError>;
    async fn get_netbird_group(&self, group_name: &netbird::GroupName) -> Result<netbird::Group, GetGroupError>;
    async fn delete_netbird_group(&self, group_id: &netbird::GroupId) -> Result<(), RequestError>;
    async fn list_setup_keys(&self) -> Result<Vec<netbird::SetupKey>, RequestError>;
    async fn get_setup_key(&self, peer_id: PeerId) -> Result<Vec<netbird::SetupKey>, RequestError>;
    async fn delete_setup_key(&self, peer_id: PeerId) -> Result<Vec<netbird::SetupKey>, RequestError>;
    #[allow(unused)] //Currently unused, but expected to be needed again
    async fn get_netbird_peer(&self, peer_id: &netbird::PeerId) -> Result<netbird::Peer, RequestError>;
    async fn delete_netbird_peer(&self, peer_id: &netbird::PeerId) -> Result<(), RequestError>;
    async fn create_netbird_self_policy(&self, group: netbird::Group, policy_name: netbird::PolicyName) -> Result<(), RequestError>;
    async fn get_netbird_policy(&self, policy_name: &netbird::PolicyName) -> Result<netbird::Policy, GetPoliciesError>;
    async fn delete_netbird_policy(&self, policy_id: &netbird::PolicyId) -> Result<(), RequestError>;
    async fn generate_netbird_setup_key(&self, peer_id: PeerId) -> Result<netbird::SetupKey, CreateSetupKeyError>;
}

pub struct DefaultClient {
    netbird_url: Url,
    setup_key_expiration: Duration,
    requester: Box<dyn RequestHandler + Send + Sync>,
}

#[derive(Serialize)]
struct CreateSetupKey {
    name: String,
    r#type: netbird::SetupKeyType,
    expires_in: u64, //seconds
    revoked: bool,
    auto_groups: Vec<String>,
    usage_limit: u64,
}

impl CreateSetupKey {
    pub fn deleted(name: String) -> Self {
        CreateSetupKey {
            name,
            r#type: netbird::SetupKeyType::Reusable,
            expires_in: 0,
            revoked: true,
            auto_groups: vec![],
            usage_limit: 0,
        }
    }
}

impl DefaultClient {
    const APPLICATION_JSON: &'static str = "application/json";

    pub async fn create(
        netbird_url: Url,
        ca: Option<&[u8]>,
        token: Option<netbird::NetbirdToken>,
        requester: Option<Box<dyn RequestHandler + Send + Sync>>,
        timeout: Duration,
        retries: u32,
        setup_key_expiration: Duration,
    ) -> Result<Self, CreateClientError>
    {
        let headers = HeaderMap::from_iter([
            (header::ACCEPT, DefaultClient::APPLICATION_JSON.parse().unwrap()),
        ]);

        let client = {
            let mut client = reqwest::Client::builder()
                .default_headers(headers)
                .https_only(true);

            if let Some(ca) = ca {
                let certificate = Certificate::from_pem(ca)
                    .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to parse ca certificate:\n  {cause}") })?;
                client = client.add_root_certificate(certificate);
            }

            client
                .tls_built_in_root_certs(true)
                .build()
                .expect("Failed to construct client.")
        };
        let reqwest_client = OidcReqwestClient::from_client(client.clone());
        opendut_util_core::testing::init_localenv_secrets();  // TODO: remove this line and properly pass secrets
        let client_config = OidcConfidentialClientConfig::new(
            ClientId::new("netbird-backend".to_string()),
            ClientSecret::new(
                std::env::var("NETBIRD_MANAGEMENT_CLIENT_SECRET")
                    .expect("NETBIRD_MANAGEMENT_CLIENT_SECRET environment variable not set in test environment")
            ),
            Url::parse("https://auth.opendut.local/realms/netbird/").unwrap(),
            vec![],
        );
        let confidential_client = ConfidentialClient::from_client_config(client_config, reqwest_client)
            .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to create confidential client:\n  {cause}") })?;

        let auth_client = ConfidentialClient::build_client_with_middleware(confidential_client.clone());
        // get self user id from NetBird
        let netbird_token = create_api_token(auth_client.clone(), &netbird_url).await?;

        let requester = requester.unwrap_or_else(|| {
            Box::new(DefaultRequestHandler::new(
                client,
                RequestHandlerConfig::new(timeout, retries),
                netbird_token,
            ))
        });

        Ok(Self {
            netbird_url,
            setup_key_expiration,
            requester,
        })
    }
}

#[async_trait]
impl Client for DefaultClient {

    #[tracing::instrument(skip(self), level="trace")]
    async fn create_netbird_group(&self, name: netbird::GroupName, peers: Vec<netbird::PeerId>) -> Result<netbird::Group, RequestError> {

        let url = routes::groups(self.netbird_url.clone());

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

    #[tracing::instrument(skip(self), level="trace")]
    async fn get_netbird_group(&self, group_name: &netbird::GroupName) -> Result<netbird::Group, GetGroupError> {
        let url = routes::groups(self.netbird_url.clone());
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

    #[tracing::instrument(skip(self), level="trace")]
    async fn delete_netbird_group(&self, group_id: &netbird::GroupId) -> Result<(), RequestError> {
        let url = routes::group(Clone::clone(&self.netbird_url), group_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        parse_response_status(response, format!("NetBird group with ID <{group_id:?}>")).await
    }

    async fn list_setup_keys(&self) -> Result<Vec<netbird::SetupKey>, RequestError> {
        let url = routes::setup_keys(Clone::clone(&self.netbird_url));

        let request = Request::new(Method::GET, url);

        let response = self.requester.handle(request).await?;

        let setup_keys = response.json::<Vec<netbird::SetupKey>>().await
            .map_err(RequestError::JsonDeserialization)?;

        Ok(setup_keys)
    }

    async fn get_setup_key(&self, peer_id: PeerId) -> Result<Vec<netbird::SetupKey>, RequestError> {
        let name = netbird::setup_key_name_format(peer_id);
        let all_keys = self.list_setup_keys().await?;
        let found_setup_keys = all_keys.into_iter().filter(|setup_key| setup_key.name.eq(&name)).collect::<Vec<_>>();
        Ok(found_setup_keys)
    }

    async fn delete_setup_key(&self, peer_id: PeerId) -> Result<Vec<netbird::SetupKey>, RequestError> {
        let found_setup_keys = self.get_setup_key(peer_id).await?;
        for setup_key in found_setup_keys.iter() {
            let url = routes::setup_key(Clone::clone(&self.netbird_url), &setup_key.id);
            let body = CreateSetupKey::deleted(setup_key.name.to_string());
            let request = put_json_request(url, body)?;
            let response = self.requester.handle(request).await?;
            response.error_for_status()
                .map_err(RequestError::IllegalStatus)?;
        }
        Ok(found_setup_keys)
    }

    #[tracing::instrument(skip(self), level="trace")]
    async fn get_netbird_peer(&self, peer_id: &netbird::PeerId) -> Result<netbird::Peer, RequestError> {
        let url = routes::peer(Clone::clone(&self.netbird_url), peer_id);

        let request = Request::new(Method::GET, url);

        let response = self.requester.handle(request).await?
            .error_for_status().map_err(RequestError::IllegalStatus)?;

        let result = response.json::<netbird::Peer>().await
            .map_err(RequestError::JsonDeserialization)?;

        Ok(result)
    }

    #[tracing::instrument(skip(self), level="trace")]
    async fn delete_netbird_peer(&self, peer_id: &netbird::PeerId) -> Result<(), RequestError> {
        let url = routes::peer(Clone::clone(&self.netbird_url), peer_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        response.error_for_status()
            .map_err(RequestError::IllegalStatus)?;

        Ok(())
    }

    #[tracing::instrument(skip(self), level="trace")]
    async fn create_netbird_self_policy(&self, group: netbird::Group, policy_name: netbird::PolicyName) -> Result<(), RequestError> {
        let url = routes::policies(self.netbird_url.clone());

        let body = {
            #[derive(Serialize, Debug)]
            struct CreateAccessControlRule {
                name: netbird::PolicyName,
                description: String,
                enabled: bool,
                action: netbird::RuleAction,
                bidirectional: bool,
                protocol: netbird::RuleProtocol,
                sources: Vec<netbird::GroupId>,
                destinations: Vec<netbird::GroupId>,
            }

            let description = policy_name.description();
            let rule = CreateAccessControlRule {
                name: policy_name.clone(),
                description: description.clone(),
                enabled: true,
                action: netbird::RuleAction::Accept,
                bidirectional: true,
                protocol: netbird::RuleProtocol::All,
                sources: vec![group.id.clone()],
                destinations: vec![group.id],
            };
            #[derive(Serialize, Debug)]
            struct CreatePolicy {
                name: netbird::PolicyName,
                description: String,
                enabled: bool,
                rules: Vec<CreateAccessControlRule>
            }
            CreatePolicy {
                name: policy_name,
                description,
                enabled: true,
                rules: vec![rule]
            }
        };
        let request = post_json_request(url, body)?;

        let response = self.requester.handle(request).await?;
        response.error_for_status()
            .map_err(RequestError::IllegalStatus)?;

        Ok(())
    }

    #[tracing::instrument(skip(self), level="trace")]
    async fn get_netbird_policy(&self, policy_name: &netbird::PolicyName) -> Result<netbird::Policy, GetPoliciesError> {
        let url = routes::policies(self.netbird_url.clone());
        let request = Request::new(Method::GET, url);
        let response = self.requester.handle(request).await
            .map_err(|cause| GetPoliciesError::RequestFailure { policy_name: policy_name.to_owned(), cause })?;
        let result = response.json::<Vec<netbird::Policy>>().await
            .map_err(|cause| GetPoliciesError::RequestFailure { policy_name: policy_name.to_owned(), cause: RequestError::JsonDeserialization(cause) })?;

        let policies = result.into_iter()
            .filter(|policy| policy.name == *policy_name)
            .collect::<Vec<_>>();

        if policies.len() > 1 {
            Err(GetPoliciesError::MultiplePoliciesFound { policy_name: policy_name.to_owned() })
        } else {
            policies.into_iter().next().ok_or(GetPoliciesError::PolicyNotFound { policy_name: policy_name.to_owned() })
        }
    }

    #[tracing::instrument(skip(self), level="trace")]
    async fn delete_netbird_policy(&self, policy_id: &netbird::PolicyId) -> Result<(), RequestError> {
        let url = routes::policy(Clone::clone(&self.netbird_url), policy_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        parse_response_status(response, format!("NetBird policy with ID <{policy_id:?}>")).await
    }

    #[tracing::instrument(skip(self), level="trace")]
    async fn generate_netbird_setup_key(&self, peer_id: PeerId) -> Result<netbird::SetupKey, CreateSetupKeyError> {
        let peer_group_name = netbird::GroupName::from(peer_id);
        let peer_group = self.get_netbird_group(&peer_group_name).await
            .map_err(|cause| CreateSetupKeyError::PeerGroupNotFound { peer_id, cause })?;

        let url = routes::setup_keys(self.netbird_url.clone());

        let body = {
            

            CreateSetupKey {
                name: netbird::setup_key_name_format(peer_id),
                r#type: netbird::SetupKeyType::Reusable,
                expires_in: self.setup_key_expiration.as_secs(),
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
}

pub(crate) fn post_json_request(url: Url, body: impl Serialize) -> Result<Request, RequestError> {
    let mut request = Request::new(Method::POST, url);

    request.headers_mut()
        .insert(header::CONTENT_TYPE, DefaultClient::APPLICATION_JSON.parse().unwrap());

    let body = serde_json::to_vec(&body)
        .map_err(RequestError::JsonSerialization)?;

    *request.body_mut() = Some(Body::from(body));

    Ok(request)
}

fn put_json_request(url: Url, body: impl Serialize) -> Result<Request, RequestError> {
    let mut request = Request::new(Method::PUT, url);

    request.headers_mut()
        .insert(header::CONTENT_TYPE, DefaultClient::APPLICATION_JSON.parse().unwrap());

    let body = serde_json::to_vec(&body)
        .map_err(RequestError::JsonSerialization)?;

    *request.body_mut() = Some(Body::from(body));

    Ok(request)
}

async fn parse_response_status(response: Response, error_text: String) -> Result<(), RequestError> {
    match response.error_for_status_ref() {
        Ok(_) => Ok(()),
        Err(status) => {
            let body = response.text().await.unwrap_or(String::from("<no body>"));
            let status_code = status.status().expect("Error should be generated from a response");
            error!("Received status code '{code}' when deleting {error_text}:\n  {body}", code=status_code, error_text=error_text, body=body);
            Err(RequestError::IllegalRequest(status_code, body))
        }
    }
}
