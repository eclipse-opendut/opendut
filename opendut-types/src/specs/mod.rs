
pub mod cluster;
pub mod parse;
pub mod peer;

use serde::Deserialize;
use strum::Display;
use uuid::Uuid;

#[derive(Debug)]
pub struct SpecificationDocument {
    pub version: String,
    pub metadata: SpecificationMetadata,
    pub spec: Specification,
}

#[derive(Debug, Deserialize)]
pub struct SpecificationMetadata {
    pub id: Uuid,
    pub name: String,
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
    PeerDescriptorSpecification(peer::PeerDescriptorSpecification),
    ClusterConfigurationSpecification(cluster::ClusterConfigurationSpecification),
}
