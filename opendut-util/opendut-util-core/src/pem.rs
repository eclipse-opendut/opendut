pub use pem::Pem;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use crate::project;
use anyhow::{anyhow, Error};
use config::Config;

const CONFIG_KEY_NETWORK_TLS_CA_CONTENT: &str = "network.tls.ca.content";
const CONFIG_KEY_DEFAULT_NETWORK_TLS_CA: &str = "network.tls.ca";
const CONFIG_KEY_OIDC_CLIENT_CA: &str = "network.oidc.client.ca";

pub trait PemFromConfig {
    fn read_from_config(config: &Config) -> anyhow::Result<Option<Pem>> {
        Self::read_from_config_with_env_fallback(
            CONFIG_KEY_NETWORK_TLS_CA_CONTENT,
            CONFIG_KEY_OIDC_CLIENT_CA,
            CONFIG_KEY_DEFAULT_NETWORK_TLS_CA,
            config
        )
    }
    fn read_from_config_with_env_fallback(
        config_key_content: &str,
        config_key_first: &str,
        config_key_second: &str,
        config: &Config
    ) -> anyhow::Result<Option<Pem>>;
    fn from_file_path(relative_file_path: &str) -> anyhow::Result<Pem>;
}

impl PemFromConfig for Pem {
    fn read_from_config_with_env_fallback(
        config_key_content: &str,
        config_key_first: &str,
        config_key_second: &str,
        config: &Config,
    ) -> anyhow::Result<Option<Pem>> {
        fn try_load_pem_from_file(config_key: &str, config: &Config) -> Option<Pem> {
            config.get_string(config_key)
                .ok()
                .filter(|string| !string.is_empty())
                .and_then(|path| project::make_path_absolute(&path).ok())
                .and_then(|pem| read_pem_from_file_path(&pem).ok())
        }
        fn load_pem_from_config_content(config_key: &str, config: &Config) -> anyhow::Result<Option<Pem>> {
            let ca_content = config.get_string(config_key).ok();
            if let Some(content) = ca_content {
                if content.is_empty() {
                    Ok(None)
                } else {
                    let pem = Pem::from_str(&content)
                        .map_err(|error| anyhow!("Could not parse CA from configuration. Error: {error}"))?;
                    Ok(Some(pem))
                }
            } else {
                Ok(None)
            }
        }

        let pem = load_pem_from_config_content(config_key_content, config)?;
        match pem {
            Some(pem) => Ok(Some(pem)),
            None => {
                let pem = [config_key_first, config_key_second]
                    .iter()
                    .find_map(|config_key| try_load_pem_from_file(config_key, config));
                Ok(pem)
            }
        }
    }

    fn from_file_path(relative_file_path: &str) -> anyhow::Result<Pem> {
        let ca_file_path = project::make_path_absolute(relative_file_path)
            .map_err(|error| anyhow!("Could not determine path for custom CA: {relative_file_path}. Error: {error}"))?;
        read_pem_from_file_path(&ca_file_path)
    }
}



pub fn read_pem_from_file_path(ca_file_path: &PathBuf) -> Result<Pem, Error> {
    let mut buffer = Vec::new();
    let ca_file_path_as_string = ca_file_path.clone().into_os_string().into_string()
        .map_err(|error| anyhow!("Could not determine CA path. Error: {error:?}"))?;
    
    let mut ca_file = File::open(ca_file_path)
        .map_err(|error| anyhow!("Could not open CA from file: {ca_file_path_as_string}. Error: {error}"))?;
    ca_file.read_to_end(&mut buffer)
        .map_err(|error| anyhow!("Could not read CA from file: {ca_file_path_as_string}. Error: {error}"))?;
    let ca_certificate = Pem::try_from(buffer.as_slice())
        .map_err(|error| anyhow!("Could not parse CA from file: {ca_file_path_as_string}. Error: {error}"))?;
    Ok(ca_certificate)
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
        let config = create_test_config(CONFIG_KEY_DEFAULT_NETWORK_TLS_CA, root_ca_path());
        let pem = Pem::read_from_config(&config)?;
        assert!(pem.is_some());
        Ok(())
    }

    #[test]
    fn test_read_pem_from_client_ca() -> anyhow::Result<()> {
        let config = create_test_config(CONFIG_KEY_OIDC_CLIENT_CA, root_ca_path());
        let pem = Pem::read_from_config(&config)?;
        assert!(pem.is_some());
        Ok(())
    }

    #[test]
    fn test_read_pem_from_client_ca_content() -> anyhow::Result<()> {
        let content = std::fs::read_to_string(root_ca_path())
            .expect("Could not read root CA file for test");
        let config = create_test_config(CONFIG_KEY_NETWORK_TLS_CA_CONTENT, content);
        let pem = Pem::read_from_config(&config)?;
        assert!(pem.is_some());
        Ok(())
    }
}
