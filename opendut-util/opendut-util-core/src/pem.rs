pub use pem::Pem;

use std::fs::File;
use std::io::Read;
use std::ops::Not;
use std::path::PathBuf;
use std::str::FromStr;
use crate::project;
use anyhow::Context;
use config::Config;
use tracing::{debug, error, trace, warn};

/// Constants for configuration keys used throughout the codebase.
pub mod config_keys {
    pub const DEFAULT_NETWORK_TLS_CA: &str = "network.tls.ca";
    pub const OIDC_CLIENT_CA: &str = "network.oidc.client.ca";
    pub const OPENTELEMETRY_CLIENT_CA: &str = "opentelemetry.client.ca";
    pub const NETWORK_TLS_CLIENT_AUTH_CERT: &str = "network.tls.client.auth.certificate";
    pub const NETWORK_TLS_CLIENT_AUTH_KEY: &str = "network.tls.client.auth.key";
}

pub trait PemFromConfig {

    /// The configuration keys are checked in order, each time checking:
    /// - Whether a configuration key with ".content" appended is present.
    ///   If so, read the PEM directly from that variable.
    /// - Whether a file is present at the path described by the configuration key.
    ///   If so, read the PEM from this file.
    ///
    /// The first configuration key that yields a certificate is what gets returned.
    /// If none match, then `Ok(None)` is returned.
    fn read_from_config_keys_with_env_fallback(
        config_keys: &[&str],
        config: &Config
    ) -> anyhow::Result<Option<Pem>>;

    fn from_file_path(relative_file_path: &str) -> anyhow::Result<Pem>;
}

impl PemFromConfig for Pem {

    fn read_from_config_keys_with_env_fallback(
        config_keys: &[&str],
        config: &Config,
    ) -> anyhow::Result<Option<Pem>> {
        fn try_load_pem_from_file(config_key: &str, config: &Config) -> Option<Pem> {
            config.get_string(config_key)
                .ok()
                .filter(|string| string.is_empty().not())
                .and_then(|path| project::make_path_absolute(&path).ok())
                .and_then(|path| {
                    read_pem_from_file_path(&path)
                        .inspect_err(|cause| error!("Error while reading PEM from {path:?}: {cause}"))
                        .ok()
                })
        }

        fn load_pem_from_config_content(config_key: &str, config: &Config) -> anyhow::Result<Option<Pem>> {
            let pem_content = config.get_string(config_key).ok();

            if let Some(content) = pem_content {
                if content.is_empty() {
                    Ok(None)
                } else {
                    let pem = Pem::from_str(&content)
                        .context(format!("Could not parse PEM from configuration key '{config_key}'"))?;
                    Ok(Some(pem))
                }
            } else {
                Ok(None)
            }
        }

        for config_key in config_keys {
            let content_config_key = format!("{config_key}.content");

            match load_pem_from_config_content(&content_config_key, config)? {
                Some(pem) => {
                    debug!("Using PEM loaded from configuration key: {content_config_key}");
                    return Ok(Some(pem));
                }
                None =>
                    if let Some(pem) = try_load_pem_from_file(config_key, config) {
                        debug!("Using PEM loaded from configuration key: {config_key}");
                        return Ok(Some(pem));
                    }
            };
        }

        warn!("No TLS keys found in configured locations: {config_keys:?}");
        Ok(None)
    }

    fn from_file_path(relative_file_path: &str) -> anyhow::Result<Pem> {
        let pem_file_path = project::make_path_absolute(relative_file_path)
            .context(format!("Could not determine path for PEM file: {relative_file_path}"))?;

        read_pem_from_file_path(&pem_file_path)
    }
}



fn read_pem_from_file_path(path: &PathBuf) -> anyhow::Result<Pem> {
    trace!("Attempting to load PEM from file: {path:?}");

    let mut file = File::open(path)
        .context(format!("Could not open PEM from file: {path:?}"))?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)
        .context(format!("Could not read PEM from file: {path:?}"))?;

    let pem = Pem::try_from(buffer.as_slice())
        .context(format!("Could not parse PEM from file: {path:?}"))?;

    trace!("Pem loaded from file: {path:?}");

    Ok(pem)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn root_ca_path() -> String {
        let path = project::make_path_absolute("resources/development/tls/insecure-development-ca.pem")
            .expect("Could not make path for custom CA");
        assert!(path.exists());
        path.into_os_string().into_string()
            .expect("Could not determine path for custom root CA")
    }

    fn create_test_config(key: &str, value: impl Into<String>) -> Config {
        Config::builder()
            .set_override(key, value.into())
            .expect("Could not set config")
            .build()
            .expect("Could not build test configuration")
    }

    #[test]
    fn test_read_pem_from_generic_ca() -> anyhow::Result<()> {
        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, root_ca_path());
        let pem = Pem::read_from_config_keys_with_env_fallback(
            &[config_keys::DEFAULT_NETWORK_TLS_CA],
            &config
        )?;
        assert!(pem.is_some());
        Ok(())
    }

    #[test]
    fn test_read_pem_from_client_ca() -> anyhow::Result<()> {
        let config = create_test_config(config_keys::OIDC_CLIENT_CA, root_ca_path());
        let pem = Pem::read_from_config_keys_with_env_fallback(
            &[config_keys::OIDC_CLIENT_CA],
            &config
        )?;
        assert!(pem.is_some());
        Ok(())
    }

    #[test]
    fn test_read_pem_from_client_ca_content() -> anyhow::Result<()> {
        let content = std::fs::read_to_string(root_ca_path())
            .expect("Could not read root CA file for test");

        let original_config_key = config_keys::DEFAULT_NETWORK_TLS_CA;
        let content_config_key = format!("{original_config_key}.content");

        let config = create_test_config(&content_config_key, content);
        let pem = Pem::read_from_config_keys_with_env_fallback(
            &[original_config_key],
            &config
        )?;
        assert!(pem.is_some());
        Ok(())
    }
}
