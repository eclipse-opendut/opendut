mod client_auth;

pub use ::pem::Pem;
pub use client_auth::ClientAuth;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::project;
use anyhow::{anyhow, Context};
use config::Config;
use tracing::{debug, error, trace, warn};

/// Constants for configuration keys used throughout the codebase.
pub mod config_keys {
    pub use super::client_auth::ClientAuthConfigKeys;
    use super::client_auth::client_auth_config_keys;

    pub const DEFAULT_NETWORK_TLS_CA: &str =             "network.tls.ca";
    pub const DEFAULT_NETWORK_TLS_CERTIFICATE: &str =    "network.tls.certificate";
    pub const DEFAULT_NETWORK_TLS_KEY: &str =            "network.tls.key";
    pub const DEFAULT_NETWORK_TLS_SERVER_AUTH_CA: &str = "network.tls.server.auth.ca";

    pub const DEFAULT_NETWORK_TLS_CLIENT_AUTH: ClientAuthConfigKeys = client_auth_config_keys!("network.tls.client.auth");

    pub const NETWORK_OIDC_CLIENT_TLS_CA: &str = "network.oidc.client.tls.ca";
    pub const NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH: ClientAuthConfigKeys = client_auth_config_keys!("network.oidc.client.tls.client.auth");

    pub const OPENTELEMETRY_TLS_CA: &str = "opentelemetry.tls.ca";
    pub const OPENTELEMETRY_TLS_CLIENT_AUTH: ClientAuthConfigKeys = client_auth_config_keys!("opentelemetry.tls.client.auth");

    pub const VPN_NETBIRD_CLIENT_TLS_CLIENT_AUTH: ClientAuthConfigKeys = client_auth_config_keys!("vpn.netbird.client.tls.client.auth");
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
    ) -> anyhow::Result<Vec<Pem>>;

    fn from_file_path(relative_file_path: &Path) -> anyhow::Result<Vec<Pem>>;
}

impl PemFromConfig for Pem {

    /// Read PEM certificate or key from config value (provided as plaintext value or file path).
    /// First try to read PEM with given config_key, then use the fallback_config_key.
    fn read_from_configured_path_or_content(
        config_key: &str,
        fallback_config_key: Option<&str>,
        config: &Config,
    ) -> anyhow::Result<Vec<Pem>> {

        let config_keys = {
            let mut config_keys = vec![config_key];
            if let Some(fallback_config_key) = fallback_config_key {
                config_keys.push(fallback_config_key);
            }
            config_keys
        };
        let pem = config_keys.iter().find_map(|config_key| {
            read_pem_from_config_key(config_key, config).ok()
        });
        match pem {
            None => {
                warn!("No TLS key/certificate found in configured locations: {config_keys:?}");
                Ok(Vec::new())
            }
            Some(pem_objects) => {
                Ok(pem_objects)
            }
        }
    }

    fn from_file_path(relative_file_path: &Path) -> anyhow::Result<Vec<Pem>> {
        let pem_file_path = project::make_path_absolute(relative_file_path)
            .context(format!("Could not determine path for PEM file: {relative_file_path:?}"))?;

        read_pem_from_file_path(&pem_file_path)
    }
}

fn read_pem_from_config_key(config_key: &str, config: &Config) -> anyhow::Result<Vec<Pem>> {

    fn try_load_pem_from_file_path(config_value: &str, config_key: &str) -> anyhow::Result<Vec<Pem>> {
        let path = project::make_path_absolute(config_value)?;
        read_pem_from_file_path(&path)
            .inspect_err(|source| {
                let mut error_message = source.to_string();
                for error in source.chain() {
                    error_message.push_str("\n    Caused by: ");
                    error_message.push_str(&error.to_string())
                }
                error!("Error while reading PEM from path {path:?} configured via configuration key '{config_key}': {error_message}")
            })
    }

    match config.get_string(config_key).ok() {
        None => Err(anyhow!("No PEM found in configuration key: {config_key}")),
        Some(config_value) if config_value.is_empty() => Err(anyhow!("No PEM found in configuration key: {config_key}")),
        Some(config_value) => {
            match read_pem_from_buffer(config_value.as_bytes(), &format!("config key={}", config_key)) {
                Ok(pems) => {
                    debug!("Using PEM loaded from text value of configuration key: {config_key}, number of PEM object(s): {}", pems.len());
                    Ok(pems)
                }
                Err(source) => {
                    if config_value.starts_with("-----BEGIN") { //very likely that user wanted to specify PEM, so return error directly
                        Err(source)
                            .context("Failed to load text value as PEM, which was configured in configuration key.")
                    }
                    else if let Ok(pem) = try_load_pem_from_file_path(&config_value, config_key) {
                        debug!("Using PEM loaded from file path defined in configuration key: {config_key}={config_value}, number of PEM object(s): {}.", pem.len());
                        Ok(pem)
                    }
                    else {
                        Err(anyhow!("No PEM found in configuration key: {config_key}"))
                    }
                }
            }
        }
    }
}


pub fn join_pem_objects(pems: &[Pem]) -> String {
    pems.iter().map(|cert| cert.to_string().replace("\r\n", "\n")).collect::<Vec<_>>().join("")
}


fn read_pem_from_file_path(path: &PathBuf) -> anyhow::Result<Vec<Pem>> {
    trace!("Attempting to load PEM from file={}", path.display());

    let mut file = File::open(path)
        .context(format!("Could not open PEM from file: {path:?}"))?;

    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)
        .context(format!("Could not read PEM from file: {path:?}"))?;

    read_pem_from_buffer(buffer.as_slice(), &format!("file={}", path.display()))
}

pub fn read_pem_from_buffer<B: AsRef<[u8]>>(input: B, source: &str) -> anyhow::Result<Vec<Pem>> {
    let pem = pem::parse_many(input)
        .context(format!("Could not parse PEM from {source}"))?;
    if pem.is_empty() {
        Err(anyhow!("No PEM found found in {source}"))
    } else {
        let num = pem.len();
        trace!("Loaded {num} PEM object(s) from {source}");
        Ok(pem)
    }
}

pub fn read_certificate_subject(pem: &Pem) -> anyhow::Result<String> {
    let cert = x509_parser::pem::Pem::iter_from_buffer(pem.to_string().as_bytes())
        .next()
        .expect("No PEM found in first certificate of test chain.")
        .context("Could not parse first PEM of test chain.")?;
    let name = cert
        .parse_x509()?
        .subject
        .to_string();
    Ok(name)
}


#[cfg(test)]
mod tests {
    use std::fs;
    use repo_path::repo_path;
    use super::*;

    #[test_log::test]
    fn should_read_pem_from_generic_ca() -> anyhow::Result<()> {
        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, root_ca_path());

        let pem = Pem::read_from_configured_path_or_content(
            config_keys::DEFAULT_NETWORK_TLS_CA,
            None,
            &config
        )?;
        assert!(!pem.is_empty());
        Ok(())
    }

    #[test_log::test]
    fn should_read_pem_from_client_ca() -> anyhow::Result<()> {
        let config = create_test_config(config_keys::NETWORK_OIDC_CLIENT_TLS_CA, root_ca_path());

        let pem = Pem::read_from_configured_path_or_content(
            config_keys::NETWORK_OIDC_CLIENT_TLS_CA,
            None,
            &config
        )?;
        assert!(!pem.is_empty());
        Ok(())
    }

    #[test_log::test]
    fn should_read_pem_from_client_ca_content() -> anyhow::Result<()> {
        let content = fs::read_to_string(root_ca_path())
            .expect("Could not read root CA file for test");

        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, content);
        let pem = Pem::read_from_configured_path_or_content(
            config_keys::DEFAULT_NETWORK_TLS_CA,
            None,
            &config
        )?;
        assert!(!pem.is_empty());
        Ok(())
    }

    #[test_log::test]
    fn should_read_pem_from_configured_text_value() -> anyhow::Result<()> {
        let pem_sample = root_ca_content();

        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, &pem_sample);

        let result = read_pem_from_config_key(config_keys::DEFAULT_NETWORK_TLS_CA, &config)?.first().cloned();

        assert_eq!(result, Some(pem::parse(pem_sample)?));
        Ok(())
    }

    #[test_log::test]
    fn should_error_when_provided_with_a_malformed_pem_value() -> anyhow::Result<()> {
        let pem_sample = root_ca_content()
            .replace("MII", "WOOHOO");

        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, pem_sample);

        let result = read_pem_from_config_key(config_keys::DEFAULT_NETWORK_TLS_CA, &config);

        assert!(result.is_err());
        Ok(())
    }

    #[test_log::test]
    fn should_read_pem_from_configured_file_path() -> anyhow::Result<()> {
        let pem_path = root_ca_path();

        let config = create_test_config(config_keys::DEFAULT_NETWORK_TLS_CA, &pem_path);

        let result = read_pem_from_config_key(config_keys::DEFAULT_NETWORK_TLS_CA, &config)?.first().cloned();

        assert_eq!(result, Some(pem::parse(root_ca_content())?));
        Ok(())
    }

    #[test_log::test]
    fn should_read_certificate_chain() -> anyhow::Result<()> {
        let content = certificate_chain_content();
        let certificates = read_pem_from_buffer(content.as_bytes(), "test-certificate-chain")?;
        assert!(!certificates.is_empty());
        assert_eq!(certificates.len(), 2);
        Ok(())
    }

    #[test_log::test]
    fn should_read_end_certificate_chain() -> anyhow::Result<()> {
        let content = certificate_chain_content();
        let certificates = read_pem_from_buffer(content.as_bytes(), "test-certificate-chain")?;
        let first = certificates.first().cloned().expect("No certificate found in test PEM chain.");

        let certificate_subject_name = read_certificate_subject(&first)?;
        assert_eq!(certificate_subject_name, "CN=opendut.local, C=XX, ST=Some-State, O=ExampleOrg");
        Ok(())
    }

    #[test_log::test]
    fn single_pem_is_not_equal_to_chain() -> anyhow::Result<()> {
        let content = certificate_chain_content();
        let pem = pem::parse(content.as_bytes())?;
        let parsed_content = pem.contents();
        assert!(parsed_content.len().lt(&content.len()));
        Ok(())
    }

    #[test_log::test]
    fn many_pem_is_equal_to_certificate_chain() -> anyhow::Result<()> {
        let content = certificate_chain_content();
        let certificates = read_pem_from_buffer(content.as_bytes(), "test-certificate-chain")?;
        let all = join_pem_objects(&certificates);
        assert_eq!(all, content);
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

    fn certificate_chain_content() -> String {
        let path = repo_path!("resources/development/tls/pem-test-loading-a-chain.pem").to_str().unwrap().to_string();
        fs::read_to_string(&path).expect("Failed to read test certificate chain")
    }
}
