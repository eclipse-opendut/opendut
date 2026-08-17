use std::fmt::Debug;

use http::StatusCode;
use reqwest::header::InvalidHeaderValue;

use opendut_model::peer::PeerId;

use crate::netbird::group::GroupName;
use crate::netbird::policies::PolicyName;

#[derive(thiserror::Error, Debug)]
pub enum GetGroupError {
    #[error("A group with name '{group_name}' does not exist!")]
    GroupNotFound { group_name: GroupName },
    #[error("Multiple groups with name '{group_name}' exist!")]
    MultipleGroupsFound { group_name: GroupName },
    #[error("Could not request group '{group_name}':\n  {source}")]
    RequestFailure {
        group_name: GroupName,
        source: RequestError
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPoliciesError {
    #[error("A policy with name '{policy_name}' does not exist!")]
    PolicyNotFound { policy_name: PolicyName },
    #[error("Multiple policies with name '{policy_name}' exist!")]
    MultiplePoliciesFound { policy_name: PolicyName },
    #[error("Could not request policy '{policy_name}:\n  {source}")]
    RequestFailure {
        policy_name: PolicyName,
        source: RequestError
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateSetupKeyError {
    #[error("Auto-assign group for peer <{peer_id}> not found for setup-key creation:\n  {source}!")]
    PeerGroupNotFound { peer_id: PeerId, source: GetGroupError },
    #[error("Could not request setup-key creation for peer <{peer_id}>:\n  {source}")]
    RequestFailure {
        peer_id: PeerId,
        source: RequestError
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("Request middleware error: {0}")]
    RequestMiddleware(reqwest_middleware::Error),
    #[error("Received status code indicating an error: {0}")]
    IllegalStatus(reqwest::Error),
    #[error("Received status code '{0}' indicating an error: {1}")]
    IllegalRequest(StatusCode, String),
    #[error("JSON deserialization error: {0}")]
    JsonDeserialization(reqwest::Error),
    #[error("JSON serialization error: {0}")]
    JsonSerialization(serde_json::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum CreateClientError {
    #[error("Invalid header: {0}")]
    InvalidHeader(InvalidHeaderValue),
    #[error("Failed to instantiated client, due to an error: {message}")]
    InstantiationFailure {
        message: String
    },
    #[error("Failed to delete default policy.")]
    DeleteDefaultPolicy(#[source] RequestError),
}
