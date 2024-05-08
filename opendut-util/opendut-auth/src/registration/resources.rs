use config::Config;
use serde::{Deserialize, Serialize};
use url::Url;

use opendut_types::resources::Id;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceHomeUrl(Url);

#[derive(thiserror::Error, Debug)]
#[error("Invalid tab identifier: {0}")]
pub struct ResourceHomeUrlError(String);


pub const KEY_NETWORK_REMOTE_HOST: &str = "network.remote.host";
pub const KEY_NETWORK_REMOTE_PORT: &str = "network.remote.port";

impl TryFrom<&Config> for ResourceHomeUrl {
    type Error = ResourceHomeUrlError;

    fn try_from(config: &Config) -> Result<Self, ResourceHomeUrlError> {
        let carl_url = {
            let host = config.get_string(KEY_NETWORK_REMOTE_HOST)
                .map_err(|error| ResourceHomeUrlError(format!("Configuration value for '{}' should be set: {}", KEY_NETWORK_REMOTE_HOST, error)))?;
            let port = config.get_int(KEY_NETWORK_REMOTE_PORT)
                .map_err(|error| ResourceHomeUrlError(format!("Configuration value for '{}' should be set: {}", KEY_NETWORK_REMOTE_PORT, error)))?;
            Url::parse(&format!("https://{host}:{port}"))
                .map_err(|error| ResourceHomeUrlError(format!("Could not create CARL URL from given host '{host}' and {port}: {error}")))?
        };
        Ok(Self(carl_url))
    }
}

impl ResourceHomeUrl {
    pub fn new(url: Url) -> Self { Self(url) }
    pub fn value(&self) -> Url {
        self.0.clone()
    }
    pub fn resource_url(&self, resource_id: Id) -> Result<Url, ResourceHomeUrlError> {
        let path = format!("/resources/{}", resource_id.value());
        self.0.join(&path)
            .map_err(|error| ResourceHomeUrlError(format!("Failed to create resource URL for resource_id='{}': {}", resource_id.value(), error)))
    }
}
