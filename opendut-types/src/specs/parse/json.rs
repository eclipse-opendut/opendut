use serde::Deserialize;
use serde_json::Value;
use crate::specs::parse::{ParseSpecificationError, ResourceKind};
use crate::specs::{self, Specification, SpecificationDocument, SpecificationMetadata, SpecificationVersion};

#[derive(Debug, Deserialize)]
pub struct JsonSpecificationDocument {
    pub kind: String,
    pub version: String,
    pub metadata: SpecificationMetadata,
    pub spec: Value,
}

impl JsonSpecificationDocument {
    /// Tries to deserialize the given string of JSON text into a [JsonSpecificationDocument].
    ///
    ///
    /// # Example
    ///
    /// ```
    /// # use opendut_types::specs::parse::json::JsonSpecificationDocument;
    /// # use serde_json::json;
    /// # fn main() -> Result<(), opendut_types::specs::parse::ParseSpecificationError> {
    /// let json = json!({
    ///   "kind": "PeerDescriptor",
    ///   "version": "v1",
    ///   "metadata": {
    ///     "id": "6d727331-8b2c-4300-abfa-45141d98c554",
    ///     "name": "PeerA"
    ///   },
    ///   "spec": {
    ///
    ///   }
    /// }).to_string();
    ///
    ///
    /// let document = JsonSpecificationDocument::try_from_json_str(&json)?;
    ///
    /// assert_eq!(document.kind, "PeerDescriptor");
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn try_from_json_str(input: &str) -> Result<Self, ParseSpecificationError> {
        serde_json::from_str::<Self>(input)
            .map_err(|cause| ParseSpecificationError::IllegalJsonSpecification { cause })
    }
}


impl TryFrom<JsonSpecificationDocument> for SpecificationDocument {
    type Error = ParseSpecificationError;
    fn try_from(doc: JsonSpecificationDocument) -> Result<Self, Self::Error> {
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
    serde_json::from_str::<ResourceKind>(s)
        .map_err(|_| ParseSpecificationError::IllegalResourceKind { kind: String::from(s) })
}

fn parse_version(s: &str) -> Result<SpecificationVersion, ParseSpecificationError> {
    serde_json::from_str::<SpecificationVersion>(s)
        .map_err(|_| ParseSpecificationError::IllegalSpecificationVersion { version: String::from(s) })
}

fn parse_spec(kind: ResourceKind, version: SpecificationVersion, spec: Value) -> Result<Specification, ParseSpecificationError> {
    match (kind, version) {
        (ResourceKind::ClusterDescriptor, SpecificationVersion::V1) => {
            let spec = serde_json::from_value::<specs::cluster::ClusterDescriptorSpecificationV1>(spec)
                .map_err(|cause| ParseSpecificationError::IllegalJsonSpecification { cause } )?;
            Ok(Specification::ClusterDescriptorSpecification(specs::cluster::ClusterDescriptorSpecification::V1(spec)))
        }
        (ResourceKind::ClusterDescriptor, _) => {
            Err(ParseSpecificationError::UnknownVersion { kind, version })
        }
        (ResourceKind::PeerDescriptor, SpecificationVersion::V1) => {
            let spec = serde_json::from_value::<specs::peer::PeerDescriptorSpecificationV1>(spec)
                .map_err(|cause| ParseSpecificationError::IllegalJsonSpecification { cause } )?;
            Ok(Specification::PeerDescriptorSpecification(specs::peer::PeerDescriptorSpecification::V1(spec)))
        }
        (ResourceKind::PeerDescriptor, _) => {
            Err(ParseSpecificationError::UnknownVersion { kind, version })
        }
    }
}
