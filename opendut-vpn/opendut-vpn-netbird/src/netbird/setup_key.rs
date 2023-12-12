use std::fmt::{Debug, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use opendut_types::peer::PeerId;

pub fn name_format(peer_id: PeerId) -> String {
    // Fixed format, do not change. Allows resolving EDGAR's NetBird peer without a mapping table.
    format!("opendut-peer-{peer_id}")
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct SetupKey {
    pub id: String,
    pub key: Uuid,
    pub name: String,
    pub expires: Timestamp,
    pub r#type: Type,
    pub valid: bool,
    pub revoked: bool,
    pub used_times: u64,
    pub last_used: Timestamp,
    pub state: State,
    pub auto_groups: Vec<String>,
    pub updated_at: Timestamp,
    pub usage_limit: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Type {
    OneOff,
    Reusable,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Valid,
    Expired,
    Revoked,
    Overused,
}

#[derive(Deserialize, PartialEq)]
#[serde(transparent)]
pub struct Timestamp {
    #[serde(with = "time::serde::rfc3339")]
    inner: time::OffsetDateTime
}
impl Debug for Timestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}
