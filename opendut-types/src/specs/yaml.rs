use serde::Deserialize;
use serde_yaml::Value;
use crate::specs::{ClusterConfigurationSpecification, ClusterConfigurationSpecificationV1, ParseSpecificationError, PeerDescriptorSpecification, PeerDescriptorSpecificationV1, Specification, SpecificationDocument, SpecificationMetadata, yaml};

#[derive(Debug, Deserialize)]
pub struct YamlSpecificationDocument {
    pub kind: String,
    pub version: String,
    pub metadata: SpecificationMetadata,
    pub spec: Value,
}

impl TryFrom<YamlSpecificationDocument> for SpecificationDocument {
    type Error = (); // TODO: Fix error

    fn try_from(doc: YamlSpecificationDocument) -> Result<Self, Self::Error> {
        let spec = parse_spec(doc.kind.as_str(), doc.version.as_str(), doc.spec)
            .map_err(|_| ())?; // TODO: Fix error type

        Ok(SpecificationDocument {
            kind: doc.kind,
            version: doc.version,
            metadata: doc.metadata,
            spec
        })
    }
}

fn parse_spec(kind: &str, version: &str, spec: Value) -> Result<Specification, ParseSpecificationError> {
    match kind { // TODO: Check version too!
        "ClusterConfiguration" => {
            let spec = serde_yaml::from_value::<ClusterConfigurationSpecificationV1>(spec)
                .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause} )?;
            Ok(Specification::ClusterConfigurationSpecification(ClusterConfigurationSpecification::V1(spec)))
        }
        "PeerDescriptor" => {
            let spec = serde_yaml::from_value::<PeerDescriptorSpecificationV1>(spec)
                .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause} )?;
            Ok(Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(spec)))
        }
        _ => {
            Err(ParseSpecificationError::UnknownResourceKind { kind: String::from(kind) })
        }
    }
}
