use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug)]
pub enum ClusterConfigurationSpecification {
    V1(ClusterConfigurationSpecificationV1)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct ClusterConfigurationSpecificationV1 {
    #[serde(default)]
    pub leader_id: Uuid,
    pub devices: Vec<Uuid>
}
