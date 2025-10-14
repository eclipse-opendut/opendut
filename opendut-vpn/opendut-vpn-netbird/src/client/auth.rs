use crate::client::post_json_request;
use crate::netbird::error::CreateClientError;
use crate::{routes, NetbirdToken};
use http::StatusCode;
use reqwest::Url;
use reqwest_middleware::ClientWithMiddleware;
use serde::{Deserialize, Serialize};
use opendut_auth::confidential::config::OidcResourceOwnerConfidentialClientConfig;

#[allow(dead_code)]
#[derive(Deserialize)]
struct NetbirdPersonalAccessToken {
    id: String,
    name: String,
    created_at: String,
    expiration_date: String,
}

#[derive(Deserialize)]
struct CreateApiTokenResponse {
    #[allow(dead_code)]
    personal_access_token: NetbirdPersonalAccessToken,
    plain_token: String,
}

#[derive(Clone)]
pub enum NetbirdAuthenticationMethod {
    /// Use OIDC to create a new API token for the user on each startup.
    CreateApiTokenWithOidc(OidcResourceOwnerConfidentialClientConfig),
    UseExistingApiToken(NetbirdToken),
    Disabled,
}

pub(crate) async fn create_api_token(client: ClientWithMiddleware, username: &str, netbird_url: &Url) -> Result<NetbirdToken, CreateClientError> {
    let user_id = get_netbird_user_id(client.clone(), username, netbird_url).await?;
    let token_name = format!("opendut-{}", uuid::Uuid::new_v4());
    let token_response = create_netbird_api_token_for_user_id(client, netbird_url, user_id, &token_name).await?;
    // TODO: remove old api keys
    Ok(NetbirdToken::new_personal_access(token_response.plain_token))
}

async fn get_netbird_user_id(client: ClientWithMiddleware, username: &str, netbird_url: &Url) -> Result<String, CreateClientError> {
    let url = routes::users(netbird_url.clone());

    #[derive(Deserialize)]
    struct NetbirdUser {
        id: String,
        name: String,
    }

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to request NetBird users: {}", cause) })?;

    if response.status() != StatusCode::OK {
        return Err(CreateClientError::InstantiationFailure { cause: String::from("Unauthorized to access NetBird users. Check your token.") })
    }
    
    let users = response
        .json::<Vec<NetbirdUser>>()
        .await
        .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to parse NetBird users response: {cause}")})?
        .into_iter()
        .filter(|user| user.name.eq(username))
        .collect::<Vec<_>>();

    match users.first() {
        Some(user) => Ok(user.id.clone()),
        None => Err(CreateClientError::InstantiationFailure { cause: String::from("No NetBird users found.") }),
    }
}

async fn create_netbird_api_token_for_user_id(client: ClientWithMiddleware, netbird_url: &Url, user_id: String, token_name: &str) -> Result<CreateApiTokenResponse, CreateClientError> {
    let url = routes::user_tokens(netbird_url.clone(), user_id.clone());
    let body = {
        #[derive(Serialize)]
        struct CreateApiToken {
            name: String,
            #[serde(rename = "expires_in")]
            expires_in_days: u64,
        }

        CreateApiToken {
            name: token_name.to_string(),
            expires_in_days: 365,
        }
    };

    let request = post_json_request(url, body)
        .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to create NetBird create API token request: {}", cause) })?;

    let response = client
        .execute(request)
        .await
        .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to request NetBird create API token: {}", cause) })?;

    if response.status() != StatusCode::OK {
        return Err(CreateClientError::InstantiationFailure { cause: format!("Failed to create NetBird API token. Status: {}", response.status()) })
    }
    let token_response = response
        .json::<CreateApiTokenResponse>()
        .await
        .map_err(|cause| CreateClientError::InstantiationFailure { cause: format!("Failed to parse NetBird create API token response: {}", cause) })?;

    Ok(token_response)
}
