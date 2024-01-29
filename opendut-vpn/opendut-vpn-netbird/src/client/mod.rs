use std::time::Duration;

use http::{header, HeaderMap, Method};
use reqwest::{Body, Request, Response, Url};
use serde::Serialize;

use opendut_types::peer::PeerId;

use crate::{netbird, routes};
use crate::client::request_handler::{DefaultRequestHandler, RequestHandler, RequestHandlerConfig};
use crate::netbird::error;
use crate::netbird::error::{CreateSetupKeyError, GetGroupError, GetRulesError, RequestError};

mod request_handler;

mod tests;

const SETUP_KEY_EXPIRY_DURATION: Duration = Duration::from_secs(24 * 60 * 60);

const APPLICATION_JSON: &str = "application/json";

pub struct Client {
    base_url: Url,
    requester: Box<dyn RequestHandler + Send + Sync>,
}

impl Client {

    pub fn create(
        base_url: Url,
        token: Option<netbird::Token>,
        requester: Option<Box<dyn RequestHandler + Send + Sync>>
    ) -> Result<Self, error::CreateClientError>
    {
        let headers = {
            let mut headers = HeaderMap::new();
            headers.append(header::ACCEPT, APPLICATION_JSON.parse().unwrap());
            if let Some(ref token) = token {
                let auth_header = token.sensitive_header()
                    .map_err(error::CreateClientError::InvalidHeader)?;
                headers.append(header::AUTHORIZATION, auth_header);
            }
            headers
        };

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .https_only(token.is_some())
            .build()
            .expect("Failed to construct client.");

        let requester = requester.unwrap_or_else(|| {
            Box::new(DefaultRequestHandler::new(
                client,
                RequestHandlerConfig::default(), //TODO pass in from the outside
            ))
        });

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

    pub async fn get_netbird_rule(&self, rule_name: &netbird::RuleName) -> Result<netbird::Rule, GetRulesError> {
        let url = routes::rules(self.base_url.clone());
        let request = Request::new(Method::GET, url);
        let response = self.requester.handle(request).await
            .map_err(|cause| GetRulesError::RequestFailure { rule_name: rule_name.to_owned(), cause })?;
        let result = response.json::<Vec<netbird::Rule>>().await
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

    pub async fn delete_netbird_group(&self, group_id: &netbird::GroupId) -> Result<(), RequestError> {
        let url = routes::group(Clone::clone(&self.base_url), group_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        parse_response_status(response, format!("NetBird group with ID <{:?}>", group_id)).await
    }

    pub async fn delete_netbird_rule(&self, rule_id: &netbird::RuleId) -> Result<(), RequestError> {
        let url = routes::rule(Clone::clone(&self.base_url), rule_id);

        let request = Request::new(Method::DELETE, url);

        let response = self.requester.handle(request).await?;

        parse_response_status(response, format!("NetBird rule with ID <{:?}>", rule_id)).await
    }

    pub async fn create_netbird_self_access_control_rule(&self, group: netbird::Group, rule_name: netbird::RuleName) -> Result<(), RequestError> {
        let url = routes::rules(self.base_url.clone());

        let body = {
            #[derive(Serialize)]
            struct CreateAccessControlRule {
                name: netbird::RuleName,
                description: String,
                disabled: bool,
                flow: netbird::RuleFlow,
                sources: Vec<netbird::GroupId>,
                destinations: Vec<netbird::GroupId>,
            }

            let description = rule_name.description();
            CreateAccessControlRule {
                name: rule_name,
                description,
                disabled: false,
                flow: netbird::RuleFlow::Bidirect,
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
                r#type: netbird::SetupKeyType,
                expires_in: u64, //seconds
                revoked: bool,
                auto_groups: Vec<String>,
                usage_limit: u64,
            }

            CreateSetupKey {
                name: netbird::setup_key_name_format(peer_id),
                r#type: netbird::SetupKeyType::Reusable,
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
        .insert(header::CONTENT_TYPE, APPLICATION_JSON.parse().unwrap());

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
            log::error!("Received status code '{code}' when deleting {error_text}:\n  {body}", code=status_code, error_text=error_text, body=body);
            Err(RequestError::IllegalRequest(status_code, body))
        }
    }
}
