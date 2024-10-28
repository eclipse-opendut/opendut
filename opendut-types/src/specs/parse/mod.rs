use serde::Deserialize;
use strum::Display;
use crate::specs::SpecificationVersion;

#[cfg(feature = "json-specs")]
pub mod json;
#[cfg(feature = "yaml-specs")]
pub mod yaml;

#[derive(Debug, thiserror::Error)]
pub enum ParseSpecificationError {
    #[error("Failed to parse specification. Kind '{kind}' is not valid!")]
    IllegalResourceKind { kind: String },
    #[error("Failed to parse specification. Version '{version}' is not valid!")]
    IllegalSpecificationVersion { version: String },
    #[error("Failed to parse specification. Unknown version '{version}' for resource specification '{kind}'")]
    UnknownVersion { kind: ResourceKind, version: SpecificationVersion },
    #[cfg(feature = "yaml-specs")]
    #[error("Failed to parse yaml specification, due to: {cause}")]
    IllegalYamlSpecification { cause: serde_yaml::Error },
    #[cfg(feature = "json-specs")]
    #[error("Failed to parse json specification, due to: {cause}")]
    IllegalJsonSpecification { cause: serde_json::Error },
}

#[derive(Clone, Copy, Debug, Deserialize, Display)]
pub enum ResourceKind {
    PeerDescriptor,
    ClusterConfiguration,
}
