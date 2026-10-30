pub mod create;
pub mod delete;
pub mod describe;
pub mod list;
pub mod apply;

use opendut_model::viper::{ViperSourceDescriptor, ViperSourceId, ViperSourceKind};
use serde::Serialize;
use url::Url;

#[derive(Serialize, Debug)]
struct SerializableViperSource {
    name: String,
    id: ViperSourceId,
    url: Url,
    kind: &'static str,
}

impl From<ViperSourceDescriptor> for SerializableViperSource {
    fn from(source: ViperSourceDescriptor) -> Self {
        Self {
            name: source.name.to_string(),
            id: source.id,
            url: source.url,
            kind: match source.kind {
                ViperSourceKind::Git => "git",
                ViperSourceKind::Http => "http",
            },
        }
    }
}
