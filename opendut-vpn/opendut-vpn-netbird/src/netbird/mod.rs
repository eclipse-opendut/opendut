use serde::{Deserialize, Serialize};

pub(crate) mod access_control;
pub(crate) mod token;
pub(crate) mod group;
pub(crate) use group::{Group, GroupName};

pub(crate) mod setup_key;
pub(crate) use setup_key::SetupKey;
pub mod error;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GroupId(pub String);

impl From<&str> for GroupId {
    fn from(value: &str) -> Self {
        GroupId(value.to_owned())
    }
}

impl From<String> for GroupId {
    fn from(value: String) -> Self {
        GroupId(value)
    }
}
