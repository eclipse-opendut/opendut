use reqwest::header::InvalidHeaderValue;
use std::fmt::Debug;
use http::StatusCode;
use opendut_types::peer::PeerId;
use crate::netbird::group::GroupName;
use crate::netbird::rules::RuleName;


#[derive(thiserror::Error, Debug)]
pub enum GetGroupError {
    #[error("A group with name '{group_name}' does not exist!")]
    GroupNotFound { group_name: GroupName },
    #[error("Multiple groups with name '{group_name}' exist!")]
    MultipleGroupsFound { group_name: GroupName },
    #[error("Could not request group '{group_name}':\n  {cause}")]
    RequestFailure {
        group_name: GroupName,
        cause: RequestError
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetRulesError {
    #[error("A rule with name '{rule_name}' does not exist!")]
    RuleNotFound { rule_name: RuleName },
    #[error("Multiple rules with name '{rule_name}' exist!")]
    MultipleRulesFound { rule_name: RuleName },
    #[error("Could not request rule '{rule_name}:\n  {cause}")]
    RequestFailure {
        rule_name: RuleName,
        cause: RequestError
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateSetupKeyError {
    #[error("Auto-assign group for peer <{peer_id}> not found for setup-key creation:\n  {cause}!")]
    PeerGroupNotFound { peer_id: PeerId, cause: GetGroupError },
    #[error("Could not request setup-key creation for peer <{peer_id}>:\n  {cause}")]
    RequestFailure {
        peer_id: PeerId,
        cause: RequestError
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RequestError {
    #[error("Request error: {0}")]
    Request(reqwest::Error), //TODO can rename to Transport?
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
    #[error("Failed to instantiated client, due to an error: {cause}")]
    InstantiationFailure {
        cause: String
    },
}
