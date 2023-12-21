use serde::{Deserialize, Serialize};

pub(crate) mod token;
pub(crate) mod group;
pub(crate) use group::{Group, GroupName};

pub(crate) mod setup_key;
pub(crate) use setup_key::SetupKey;
pub mod error;
pub(crate) mod rules;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PeerId(pub String);
