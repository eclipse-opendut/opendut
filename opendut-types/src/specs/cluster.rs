use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug)]
pub enum ClusterDescriptorSpecification {
    V1(ClusterDescriptorSpecificationV1)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct ClusterDescriptorSpecificationV1 {
    #[serde(default)]
    pub leader_id: Uuid,
    pub devices: Vec<Uuid>
}
