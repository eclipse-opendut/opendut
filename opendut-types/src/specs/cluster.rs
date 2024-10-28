use serde::Deserialize;

#[derive(Debug)]
pub enum ClusterConfigurationSpecification {
    V1(ClusterConfigurationSpecificationV1)
}

#[derive(Debug, Deserialize)]
pub struct ClusterConfigurationSpecificationV1 {
    #[serde(default)]
    pub description: String,
}
