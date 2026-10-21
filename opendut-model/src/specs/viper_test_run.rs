use std::collections::HashMap;

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug)]
pub enum ViperTestRunDescriptorSpecification {
    V1(ViperTestRunDescriptorSpecificationV1),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ViperTestRunDescriptorSpecificationV1 {
    pub source_id: Uuid,
    pub peer_id: Uuid,
    #[serde(default)]
    pub parameters: HashMap<String, ViperParameterValueSpecificationV1>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ViperParameterValueSpecificationV1 {
    Boolean(bool),
    Number(i64),
    Text(String),
}
