use serde::Deserialize;
use serde_yaml::Value;

use crate::specs::*;

use parse::ResourceKind;
use crate::specs::parse::ParseSpecificationError;

#[derive(Debug, Deserialize)]
pub struct YamlSpecificationDocument {
    pub kind: String,
    pub version: String,
    pub metadata: SpecificationMetadata,
    pub spec: Value,
}

impl YamlSpecificationDocument {

    /// Tries to deserialize the given string of YAML text into a [YamlSpecificationDocument].
    ///
    /// In contrast to [`YamlSpecificationFile::try_from_yaml_str`] this function
    /// accepts a string containing only a single YAML document.
    ///
    /// # Example
    ///
    /// ```
    /// # use opendut_types::specs::parse::yaml::YamlSpecificationDocument;
    /// # fn main() -> Result<(), opendut_types::specs::parse::ParseSpecificationError> {
    /// let yaml = r#"
    /// kind: PeerDescriptor
    /// version: v1
    /// metadata:
    ///     id: 6d727331-8b2c-4300-abfa-45141d98c554
    ///     name: PeerA
    /// spec:
    ///     ...
    /// "#;
    ///
    /// let document = YamlSpecificationDocument::try_from_yaml_str(yaml)?;
    ///
    /// assert_eq!(document.kind, "PeerDescriptor");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_yaml_str(s: &str) -> Result<Self, ParseSpecificationError> {
        serde_yaml::from_str::<YamlSpecificationDocument>(s)
            .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause })
    }
}

impl TryFrom<YamlSpecificationDocument> for SpecificationDocument {
    type Error = ParseSpecificationError;
    fn try_from(doc: YamlSpecificationDocument) -> Result<Self, Self::Error> {
        let kind = parse_kind(doc.kind.as_str())?;
        let version = parse_version(doc.version.as_str())?;
        let spec = parse_spec(kind, version, doc.spec)?;
        Ok(SpecificationDocument {
            version: doc.version,
            metadata: doc.metadata,
            spec
        })
    }
}

fn parse_kind(s: &str) -> Result<ResourceKind, ParseSpecificationError> {
    serde_yaml::from_str::<ResourceKind>(s)
        .map_err(|_| ParseSpecificationError::IllegalResourceKind { kind: String::from(s) })
}

fn parse_version(s: &str) -> Result<SpecificationVersion, ParseSpecificationError> {
    match serde_yaml::from_str::<SpecificationVersion>(s) {
        Ok(version) => {
            match version {
                SpecificationVersion::V1 => { Ok(SpecificationVersion::V1) }
                SpecificationVersion::V2 => { Ok(SpecificationVersion::V2) }
            }
        }
        Err(_) => {
            Err(ParseSpecificationError::IllegalSpecificationVersion { version: String::from(s) })
        }
    }
}

fn parse_spec(kind: ResourceKind, version: SpecificationVersion, spec: Value) -> Result<Specification, ParseSpecificationError> {
    match (kind, version) {
        (ResourceKind::ClusterConfiguration, SpecificationVersion::V1) => {
            let spec = serde_yaml::from_value::<cluster::ClusterConfigurationSpecificationV1>(spec)
                .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause } )?;
            Ok(Specification::ClusterConfigurationSpecification(cluster::ClusterConfigurationSpecification::V1(spec)))
        }
        (ResourceKind::ClusterConfiguration, _) => {
            Err(ParseSpecificationError::UnknownVersion { kind, version })
        }
        (ResourceKind::PeerDescriptor, SpecificationVersion::V1) => {
            let spec = serde_yaml::from_value::<peer::PeerDescriptorSpecificationV1>(spec)
                .map_err(|cause| ParseSpecificationError::IllegalYamlSpecification { cause } )?;
            Ok(Specification::PeerDescriptorSpecification(peer::PeerDescriptorSpecification::V1(spec)))
        }
        (ResourceKind::PeerDescriptor, _) => {
            Err(ParseSpecificationError::UnknownVersion { kind, version })
        }
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use googletest::prelude::*;

    use super::*;

    #[test]
    pub fn test_try_from_yaml_str() -> Result<()> {
        let result = YamlSpecificationDocument::try_from_yaml_str(r#"
            kind: SomeKind
            version: MustBeAVeryHighVersionNumber
            metadata:
              id: 140f29fd-336b-48f7-9936-6b1892574543
              name: TheForgottenName
            spec:
              ...
        "#);

        verify_that!(result, ok(anything()))?;

        Ok(())
    }
    
    #[test]
    pub fn test_try_from() -> Result<()> {
         let document_string = YamlSpecificationDocument::try_from_yaml_str(r#"
            kind: PeerDescriptor
            version: v1
            metadata:
              id: 140f29fd-336b-48f7-9936-6b1892574543
              name: TheForgottenName
            spec:
              location: Ulm 
              network: 
                interfaces:
                - id: a4a3c74c-71e5-49ea-9c2e-afb387951970
                  name: eth0
                  kind: ethernet
        "#)?;
        
        let result = SpecificationDocument::try_from(document_string);
        verify_that!(result, ok(anything()))?;
        
        Ok(())
    }
    
    #[test]
    pub fn test_failing_try_from_with_illegal_resource_kind() -> Result<()> {
         let document_string = YamlSpecificationDocument::try_from_yaml_str(r#"
            kind: SomeKind
            version: v1
            metadata:
              id: 140f29fd-336b-48f7-9936-6b1892574543
              name: TheForgottenName
            spec:
              location: Ulm 
              network: 
                interfaces:
                - id: a4a3c74c-71e5-49ea-9c2e-afb387951970
                  name: eth0
                  kind: ethernet
        "#)?;
        
        let result = SpecificationDocument::try_from(document_string);
        verify_that!(result, err(anything()))?;
        
        Ok(())
    }    
    
    #[test]
    pub fn test_failing_try_from_with_unknown_version() -> Result<()> {
         let document_string = YamlSpecificationDocument::try_from_yaml_str(r#"
            kind: PeerDescriptor
            version: SomeKind
            metadata:
              id: 140f29fd-336b-48f7-9936-6b1892574543
              name: TheForgottenName
            spec:
              location: Ulm 
              network: 
                interfaces:
                - id: a4a3c74c-71e5-49ea-9c2e-afb387951970
                  name: eth0
                  kind: ethernet
        "#)?;
        
        let result = SpecificationDocument::try_from(document_string);
        verify_that!(result, err(anything()))?;
        
        Ok(())
    }
}
