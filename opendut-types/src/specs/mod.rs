
#[cfg(feature = "yaml-specs")]
pub mod yaml;

use serde::Deserialize;
use uuid::Uuid;


#[derive(Debug)]
pub enum Specification {
    PeerDescriptorSpecification(PeerDescriptorSpecification),
    ClusterConfigurationSpecification(ClusterConfigurationSpecification),
}

impl Specification {

    #[cfg(feature = "yaml-specs")]
    pub fn from_yaml_str(s: &str) -> Vec<Result<Specification, ParseSpecificationError>> {
        let mut result = Vec::new();
        for document in serde_yaml::Deserializer::from_str(s) {
            println!("2");
            result.push(serde_yaml::Value::deserialize(document)
                .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause })
                .and_then(|value| {
                    println!("3");
                    Specification::from_yaml_value(value)
                }))
        }

        result
    }

    #[cfg(feature = "yaml-specs")]
    pub fn from_yaml_value(value: serde_yaml::Value) -> Result<Specification, ParseSpecificationError> {
        let document = serde_yaml::from_value::<yaml::YamlSpecificationDocument>(value)
            .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause} )?;
        Self::from_yaml_document(document)
    }

    #[cfg(feature = "yaml-specs")]
    pub fn from_yaml_document(document: yaml::YamlSpecificationDocument) -> Result<Specification, ParseSpecificationError> {
        match document.kind.as_str() { // TODO: Check version too!
            "ClusterConfiguration" => {
                let spec = serde_yaml::from_value::<ClusterConfigurationSpecificationV1>(document.spec)
                    .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause} )?;
                Ok(Specification::ClusterConfigurationSpecification(ClusterConfigurationSpecification::V1(spec)))
            }
            "PeerDescriptor" => {
                let spec = serde_yaml::from_value::<PeerDescriptorSpecificationV1>(document.spec)
                    .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause} )?;
                Ok(Specification::PeerDescriptorSpecification(PeerDescriptorSpecification::V1(spec)))
            }
            _ => {
                Err(ParseSpecificationError::UnknownResourceKind { kind: document.kind })
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseSpecificationError {
    #[error("Unknown kind '{kind}'!")]
    UnknownResourceKind { kind: String },
    #[error("Unknown version '{version}' for resource '{kind}'")]
    UnknownVersion { kind: String, version: String },
    #[cfg(feature = "yaml-specs")]
    #[error("Failed to parse yaml specification, due to: {cause}")]
    IllegalYamlSpecification { cause: serde_yaml::Error },
}

#[derive(Debug, Deserialize)]
pub struct SpecificationMetadata {
    id: Uuid,
    name: String,
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
    location: String,
}

#[derive(Debug, Deserialize)]
pub struct ClusterConfigurationSpecificationV1 {
    #[serde(default)]
    description: String,
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use googletest::prelude::*;
    use rstest::rstest;

    use super::*;

    //#[rstest]
    //pub fn test_parse_ClusterConfiguration() -> Result<()> {
//
    //    let result = Specification::from_yaml_str(r#"
    //        ---
    //        kind: ClusterConfiguration
    //        version: v1
    //        metadata:
    //            id: 140f29fd-336b-48f7-9936-6b1892574543
    //            name: MyConfig
    //        spec:
    //            location: Ulm
    //    "#);
//
    //    println!("{result:?}");
    //    verify_that!(result, elements_are![ok(anything())])?;
//
    //    Ok(())
    //}

    #[rstest]
    pub fn test_parse_PeerDescriptor() -> Result<()> {

        let result = Specification::from_yaml_str(r#"
            version: v1
            kind: PeerDescriptor
            metadata:
              id: 140f29fd-336b-48f7-9936-6b1892574543
              name: TestPeer
            spec:
              location: Ulm
        "#);

        verify_that!(result, elements_are![ok(anything())])?;

        Ok(())
    }

    #[rstest]
    pub fn test_failing_ClusterConfiguration() -> Result<()> {
        let content = Specification::from_yaml_str(r#"
            version: v1
            kind: ClusterConfiguration
            spec:
              location: Ulm
        "#);

        verify_that!(content, elements_are![err(matches_pattern!(ParseSpecificationError::IllegalYamlSpecification { .. }))])?;

        Ok(())
    }
}
