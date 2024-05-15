use serde::Deserialize;
use serde_yaml::Value;
use crate::specs::SpecificationMetadata;

#[derive(Debug, Deserialize)]
pub struct YamlSpecificationDocument {
    pub kind: String,
    pub version: String,
    pub metadata: SpecificationMetadata,
    pub spec: Value,
}
