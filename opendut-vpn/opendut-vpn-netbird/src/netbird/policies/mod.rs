use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use opendut_types::cluster::ClusterId;

use crate::netbird::group::GroupId;

#[derive(thiserror::Error, Debug)]
#[error("Cannot create PolicyName from '{value}':\n  {cause}")]
pub struct InvalidPolicyNameError {
    value: String,
    cause: Box<dyn Error>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum PolicyName {
    Cluster(ClusterId),
    Other(String),
}

impl PolicyName {
    const CLUSTER_POLICY_PREFIX: &'static str = "opendut-cluster-policy-";

    pub fn description(&self) -> String {
        match self {
            PolicyName::Cluster(cluster_id) => format!("Policy for the openDuT cluster <{cluster_id}>."),
            PolicyName::Other(name) => name.to_owned(),
        }
    }
}

impl From<ClusterId> for PolicyName {
    fn from(cluster_id: ClusterId) -> Self {
        PolicyName::Cluster(cluster_id)
    }
}

impl TryFrom<&str> for PolicyName {
    type Error = InvalidPolicyNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(uuid) = value.strip_prefix(PolicyName::CLUSTER_POLICY_PREFIX) {
            ClusterId::try_from(uuid)
                .map(Self::Cluster)
                .map_err(|cause| InvalidPolicyNameError { value: value.to_owned(), cause: cause.into() })
        }
        else {
            Ok(Self::Other(value.to_owned()))
        }
    }
}

impl TryFrom<String> for PolicyName {

    type Error = InvalidPolicyNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PolicyName::try_from(value.as_str())
    }
}

impl From<&PolicyName> for String {
    fn from(value: &PolicyName) -> Self {
        match value {
            PolicyName::Cluster(id) => format!("{}{}", PolicyName::CLUSTER_POLICY_PREFIX, id),
            PolicyName::Other(name) => name.to_owned(),
        }
    }
}

impl From<PolicyName> for String {
    fn from(value: PolicyName) -> Self {
        String::from(&value)
    }
}

impl Display for PolicyName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PolicyId(pub String);

impl From<&str> for PolicyId {
    fn from(value: &str) -> Self {
        PolicyId(value.to_owned())
    }
}

impl From<String> for PolicyId {
    fn from(value: String) -> Self {
        PolicyId(value)
    }
}


#[derive(Debug, Deserialize)]
pub struct Policy {
    pub id: PolicyId,
    pub name: PolicyName,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
pub(crate) enum RuleAction {
    Accept,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub(crate) enum RuleProtocol {
    Tcp,
    Udp,
    All,
    Icmp,
}
