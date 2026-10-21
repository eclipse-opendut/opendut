use serde::Deserialize;
use url::Url;

#[derive(Debug)]
pub enum ViperSourceDescriptorSpecification {
    V1(ViperSourceDescriptorSpecificationV1),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ViperSourceDescriptorSpecificationV1 {
    pub url: Url,
    pub kind: ViperSourceKindSpecificationV1,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ViperSourceKindSpecificationV1 {
    Git,
    Http,
}
