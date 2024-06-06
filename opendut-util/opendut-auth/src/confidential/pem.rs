use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use config::Config;
use pem::Pem;
use opendut_util_core::project;
use crate::confidential::error::OidcClientError;

pub trait PemFromConfig {
    fn from_config_path(config_key: &str, config: &Config) -> impl std::future::Future<Output=Result<Pem, OidcClientError>> + Send;
    fn from_file_path(relative_file_path: &str) -> impl std::future::Future<Output=Result<Pem, OidcClientError>> + Send;
    fn from_file_path_sync(relative_file_path: &str) -> Result<Pem, OidcClientError>;
}

impl PemFromConfig for Pem {
    async fn from_config_path(config_key: &str, config: &Config) -> Result<Pem, OidcClientError> {
        let config_ca_file_path = config.get_string(config_key)
            .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not parse configuration field {} to load custom CA. Error: {}", config_key, error)))?;
        let ca_file_path = project::make_path_absolute(config_ca_file_path.clone())
            .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not determine path for custom CA: {}. Error: {}", config_ca_file_path, error)))?;
        read_pem_from_file_path(&ca_file_path)
    }

    async fn from_file_path(relative_file_path: &str) -> Result<Pem, OidcClientError> {
        let ca_file_path = project::make_path_absolute(relative_file_path)
            .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not determine path for custom CA: {}. Error: {}", relative_file_path, error)))?;
        read_pem_from_file_path(&ca_file_path)
    }

    fn from_file_path_sync(relative_file_path: &str) -> Result<Pem, OidcClientError> {
        let ca_file_path = project::make_path_absolute(relative_file_path)
            .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not determine path for custom CA: {}. Error: {}", relative_file_path, error)))?;
        read_pem_from_file_path(&ca_file_path)
    }
}

pub fn read_pem_from_file_path(ca_file_path: &PathBuf) -> Result<Pem, OidcClientError> {
    let mut buffer = Vec::new();
    let ca_file_path_as_string = ca_file_path.clone().into_os_string().into_string()
        .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not determine CA path. Error: {:?}", error)))?;
    
    let mut ca_file = File::open(ca_file_path)
        .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not open CA from file: {}. Error: {}", ca_file_path_as_string, error)))?;
    ca_file.read_to_end(&mut buffer)
        .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not read CA from file: {}. Error: {}", ca_file_path_as_string, error)))?;
    let ca_certificate = Pem::try_from(buffer.as_slice())
        .map_err(|error| OidcClientError::LoadCustomCA(format!("Could not parse CA from file: {}. Error: {}", ca_file_path_as_string, error)))?;
    Ok(ca_certificate)
}
