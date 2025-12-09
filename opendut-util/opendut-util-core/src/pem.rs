pub use pem::Pem;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::project;
use anyhow::Context;
use config::Config;
use tracing::{debug, error, trace, warn};

/// Constants for configuration keys used throughout the codebase.
pub mod config_keys {
    pub const DEFAULT_NETWORK_TLS_CA: &str = "network.tls.ca";
    pub const DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED: &str = "network.tls.client.auth.enabled";
    pub const DEFAULT_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE: &str = "network.tls.client.auth.certificate";
    pub const DEFAULT_NETWORK_TLS_CLIENT_AUTH_KEY: &str = "network.tls.client.auth.key";
    pub const OIDC_CLIENT_CA: &str = "network.oidc.client.ca";
    pub const OPENTELEMETRY_TLS_CA: &str = "opentelemetry.tls.ca";
    pub const OPENTELEMETRY_TLS_CLIENT_AUTH_CERTIFICATE: &str = "opentelemetry.tls.client.auth.certificate";
    pub const OPENTELEMETRY_TLS_CLIENT_AUTH_KEY: &str = "opentelemetry.tls.client.auth.key";
}

pub trait PemFromConfig {

    /// Check whether the configuration key specifies:
    /// - a text that can be parsed as a PEM. If so, read the PEM directly from that.
    /// - an existing file path. If so, read the PEM from the path.
    ///
    /// Do the same for the fallback_config_key.
    /// If none of these checks yield a certificate, `Ok(None)` is returned.
    fn read_from_configured_path_or_content(
        config_key: &str,
        fallback_config_key: Option<&str>,
        config: &Config,
    ) -> anyhow::Result<Option<Pem>>;

    fn from_file_path(relative_file_path: &Path) -> anyhow::Result<Pem>;
}

impl PemFromConfig for Pem {

    fn read_from_configured_path_or_content(
        config_key: &str,
        fallback_config_key: Option<&str>,
        config: &Config,
    ) -> anyhow::Result<Option<Pem>> {

        let config_keys = {
            let mut config_keys = vec![config_key];
            if let Some(fallback_config_key) = fallback_config_key {
                config_keys.push(fallback_config_key);
            }
            config_keys
        };

        for config_key in &config_keys {
            if let Some(pem) = read_pem_from_config_key(config_key, config)? {
                return Ok(Some(pem));
            }
        }

        warn!("No TLS keys found in configured locations: {config_keys:?}");
        Ok(None)
    }

    fn from_file_path(relative_file_path: &Path) -> anyhow::Result<Pem> {
        let pem_file_path = project::make_path_absolute(relative_file_path)
            .context(format!("Could not determine path for PEM file: {relative_file_path:?}"))?;

        read_pem_from_file_path(&pem_file_path)
    }
}

fn read_pem_from_config_key(config_key: &str, config: &Config) -> anyhow::Result<Option<Pem>> {

    fn try_load_pem_from_file_path(config_value: &str, config_key: &str) -> Option<Pem> {
        let path = project::make_path_absolute(config_value)
            .ok()?;

        read_pem_from_file_path(&path)
            .inspect_err(|cause| error!("Error while reading PEM from path {path:?} configured via configuration key '{config_key}': {cause}"))
            .ok()
    }

    let result =
        match config.get_string(config_key).ok() {
            None => None,
            Some(config_value) if config_value.is_empty() => None,
            Some(config_value) => {
                match Pem::from_str(&config_value) {
                    Ok(pem) => {
                        debug!("Using PEM loaded from text value of configuration key: {config_key}");
                        Some(pem)
                    }
                    Err(cause) => {
                        if config_value.starts_with("-----BEGIN") { //very likely that user wanted to specify PEM, so return error directly
                            return Err(cause)
                                .context("Failed to load text value as PEM, which was configured in configuration key '{config_key}'");
                        }
                        else if let Some(pem) = try_load_pem_from_file_path(&config_value, config_key) {
                            debug!("Using PEM loaded from file path defined in configuration key: {config_key}");
                            Some(pem)
                        }
                        else {
                            None
                        }
                    }
                }
            }
        };

    Ok(result)
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

    trace!("PEM loaded from file: {path:?}");

    Ok(pem)
}


#[cfg(test)]
mod tests {
    use std::fs;
    use repo_path::repo_path;
    use super::*;

    #[test]
    fn should_read_pem_from_generic_ca() -> anyhow::Result<()> {
        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, root_ca_path());

        let pem = Pem::read_from_configured_path_or_content(
            config_keys::DEFAULT_NETWORK_TLS_CA,
            None,
            &config
        )?;
        assert!(pem.is_some());
        Ok(())
    }

    #[test]
    fn should_read_pem_from_client_ca() -> anyhow::Result<()> {
        let config = create_test_config(config_keys::OIDC_CLIENT_CA, root_ca_path());

        let pem = Pem::read_from_configured_path_or_content(
            config_keys::OIDC_CLIENT_CA,
            None,
            &config
        )?;
        assert!(pem.is_some());
        Ok(())
    }

    #[test]
    fn should_read_pem_from_client_ca_content() -> anyhow::Result<()> {
        let content = fs::read_to_string(root_ca_path())
            .expect("Could not read root CA file for test");

        let config = create_test_config(&config_keys::DEFAULT_NETWORK_TLS_CA, content);
        let pem = Pem::read_from_configured_path_or_content(
            config_keys::DEFAULT_NETWORK_TLS_CA,
            None,
            &config
        )?;
        assert!(pem.is_some());
        Ok(())
    }

    #[test]
    fn should_read_pem_from_configured_text_value() -> anyhow::Result<()> {
        let pem_sample = root_ca_content();

        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, &pem_sample);

        let result = read_pem_from_config_key(config_keys::DEFAULT_NETWORK_TLS_CA, &config)?;

        assert_eq!(result, Some(pem::parse(pem_sample)?));
        Ok(())
    }

    #[test]
    fn should_error_when_provided_with_a_malformed_pem_value() -> anyhow::Result<()> {
        let pem_sample = root_ca_content()
            .replace("MII", "WOOHOO");

        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, pem_sample);

        let result = read_pem_from_config_key(config_keys::DEFAULT_NETWORK_TLS_CA, &config);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn should_read_pem_from_configured_file_path() -> anyhow::Result<()> {
        let pem_path = root_ca_path();

        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, &pem_path);

        let result = read_pem_from_config_key(config_keys::DEFAULT_NETWORK_TLS_CA, &config)?;

        assert_eq!(result, Some(pem::parse(root_ca_content())?));
        Ok(())
    }


    fn create_test_config(key: &str, value: impl Into<String>) -> Config {
        Config::builder()
            .set_override(key, value.into())
            .expect("Could not set config")
            .build()
            .expect("Could not build test configuration")
    }

    fn root_ca_path() -> String {
        repo_path!("resources/development/tls/insecure-development-ca.pem")
            .to_str().unwrap().to_string()
    }

    fn root_ca_content() -> String {
        fs::read_to_string(root_ca_path())
            .expect("Failed to read test certificate")
    }
}
