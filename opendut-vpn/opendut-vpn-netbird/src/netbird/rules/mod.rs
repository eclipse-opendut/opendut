use std::error::Error;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use opendut_types::cluster::ClusterId;

use crate::netbird::group::GroupId;

#[derive(thiserror::Error, Debug)]
#[error("Cannot create RuleName from '{value}':\n  {cause}")]
pub struct InvalidRuleNameError {
    value: String,
    cause: Box<dyn Error>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum RuleName {
    Cluster(ClusterId),
    Other(String),
}

impl RuleName {
    const CLUSTER_RULE_PREFIX: &'static str = "opendut-cluster-rule-";

    pub fn description(&self) -> String {
        match self {
            RuleName::Cluster(cluster_id) => format!("Rule for the openDuT cluster <{cluster_id}>."),
            RuleName::Other(name) => name.to_owned(),
        }
    }
}

impl From<ClusterId> for RuleName {
    fn from(cluster_id: ClusterId) -> Self {
        RuleName::Cluster(cluster_id)
    }
}

impl TryFrom<&str> for RuleName {
    type Error = InvalidRuleNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(uuid) = value.strip_prefix(RuleName::CLUSTER_RULE_PREFIX) {
            ClusterId::try_from(uuid)
                .map(Self::Cluster)
                .map_err(|cause| InvalidRuleNameError { value: value.to_owned(), cause: cause.into() })
        }
        else {
            Ok(Self::Other(value.to_owned()))
        }
    }
}

impl TryFrom<String> for RuleName {

    type Error = InvalidRuleNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        RuleName::try_from(value.as_str())
    }
}

impl From<&RuleName> for String {
    fn from(value: &RuleName) -> Self {
        match value {
            RuleName::Cluster(id) => format!("{}{}", RuleName::CLUSTER_RULE_PREFIX, id),
            RuleName::Other(name) => name.to_owned(),
        }
    }
}

impl From<RuleName> for String {
    fn from(value: RuleName) -> Self {
        String::from(&value)
    }
}

impl Display for RuleName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleId(pub String);

impl From<&str> for RuleId {
    fn from(value: &str) -> Self {
        RuleId(value.to_owned())
    }
}

impl From<String> for RuleId {
    fn from(value: String) -> Self {
        RuleId(value)
    }
}


#[derive(Debug, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: RuleName,
    pub description: String,
    pub disabled: bool,
    pub flow: RuleFlow,

    pub sources: Vec<GroupInfo>,
    pub destinations: Vec<GroupInfo>,
}


#[derive(Debug, Deserialize)]
pub struct GroupInfo {
    pub id: GroupId,
    pub name: String,
    pub peers_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="kebab-case")]
pub(crate) enum RuleFlow {
    Bidirect,
}
