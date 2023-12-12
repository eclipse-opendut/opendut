use serde::Deserialize;

pub use group_name::GroupName;
use crate::netbird;

mod group_name;

#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: netbird::GroupId,
    pub name: GroupName,
    pub peers_count: usize,
    #[serde(deserialize_with = "opendut_util::serde::deserialize_null_default")]
    pub peers: Vec<GroupPeerInfo>,
    // fields omitted
}

#[derive(Debug, Deserialize)]
pub struct GroupPeerInfo {
    pub id: netbird::PeerId,
    pub name: String,
}
