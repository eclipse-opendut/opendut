
#[cfg(feature = "yaml-specs")]
pub mod yaml;
#[cfg(feature = "json-specs")]
pub mod json;

use serde::Deserialize;
use strum::Display;
use uuid::Uuid;

#[derive(Debug)]
pub struct SpecificationDocument {
    pub kind: ResourceKind,
    pub version: String,
    pub metadata: SpecificationMetadata,
    pub spec: Specification,
}

#[derive(Clone, Copy, Debug, Deserialize, Display)]
pub enum ResourceKind {
    PeerDescriptor,
    ClusterConfiguration,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Display)]
#[serde(rename_all = "camelCase")]
pub enum SpecificationVersion {
    V1,
    V2,
}

#[derive(Debug)]
pub enum Specification {
    PeerDescriptorSpecification(PeerDescriptorSpecification),
    ClusterConfigurationSpecification(ClusterConfigurationSpecification),
}

impl Specification {
    
    #[cfg(feature = "json-specs")]
    pub fn from_json_str(s: &str) -> Result<Specification, ParseSpecificationError> {
        let document = serde_json::from_str::<json::JsonSpecificationDocument>(s).map_err(|cause| ParseSpecificationError::IllegalJsonSpecification { cause })?;
        Specification::from_json_value(document.spec)
        
    }
    
    #[cfg(feature = "json-specs")]
    pub fn from_json_value(value: serde_json::Value) -> Result<Specification, ParseSpecificationError> {
        let document = serde_json::from_value::<json::JsonSpecificationDocument>(value)
            .map_err(|cause| ParseSpecificationError::IllegalJsonSpecification { cause })?;
        Self::from_json_document(document)
    }

    #[cfg(feature = "json-specs")]
    pub fn from_json_document(document: json::JsonSpecificationDocument) -> Result<Specification, ParseSpecificationError> {
        match document.kind.as_str() { // TODO: Check version too!
            "ClusterConfiguration" => {
                let spec = serde_json::from_value::<ClusterConfigurationSpecificationV1>(document.spec)
                    .map_err(|cause| ParseSpecificationError::IllegalJsonSpecification { cause} )?;
                Ok(Specification::ClusterConfigurationSpecification(ClusterConfigurationSpecification::V1(spec)))
            }
            "PeerDescriptor" => {
                let spec = serde_json::from_value::<PeerDescriptorSpecificationV1>(document.spec)
                    .map_err(|cause| ParseSpecificationError::IllegalJsonSpecification { cause} )?;
                Ok(Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(spec)))
            }
            _ => {
                Err(ParseSpecificationError::IllegalResourceKind { kind: document.kind })
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseSpecificationError {
    #[error("Kind '{kind}' is not valid!")]
    IllegalResourceKind { kind: String },
    #[error("Version '{version}' is not valid!")]
    IllegalSpecificationVersion { version: String },
    #[error("Unknown version '{version}' for resource specification '{kind}'")]
    UnknownVersion { kind: ResourceKind, version: SpecificationVersion },
    #[cfg(feature = "yaml-specs")]
    #[error("Failed to parse yaml specification, due to: {cause}")]
    IllegalYamlSpecification { cause: serde_yaml::Error },
    #[cfg(feature = "json-specs")]
    #[error("Failed to parse json specification, due to: {cause}")]
    IllegalJsonSpecification { cause: serde_json::Error },
}

#[derive(Debug, Deserialize)]
pub struct SpecificationMetadata {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug)]
pub enum PeerDescriptorSpecification {
    V1(PeerDescriptorSpecificationV1)
}

#[derive(Debug)]
pub enum ClusterConfigurationSpecification {
    V1(ClusterConfigurationSpecificationV1)
}

#[derive(Debug, Deserialize)]
pub struct PeerDescriptorSpecificationV1 {
    #[serde(default)]
    pub location: String,
}

#[derive(Debug, Deserialize)]
pub struct ClusterConfigurationSpecificationV1 {
    #[serde(default)]
    pub description: String,
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {

}
