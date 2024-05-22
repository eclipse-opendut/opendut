use serde::Deserialize;
use serde_json::Value;
use crate::specs::SpecificationMetadata;

#[derive(Debug, Deserialize)]
pub struct JsonSpecificationDocument {
    pub kind: String,
    pub version: String,
    pub metadata: SpecificationMetadata,
    pub spec: Value,
}
