use serde::{Deserialize, Serialize};

pub use group_name::GroupName;

use crate::netbird;

mod group_name;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: GroupName,
    pub peers_count: usize,
    #[serde(deserialize_with = "opendut_util::serde::deserialize_null_default")]
    pub peers: Vec<GroupPeerInfo>,
    // fields omitted
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GroupPeerInfo {
    pub id: netbird::PeerId,
    pub name: String,
}

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
