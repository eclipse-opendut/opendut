use anyhow::{anyhow, Context};
use axum_server::tls_rustls::RustlsConfig;
use config::Config;
use opendut_util::project;
use std::fs;
use tracing::debug;

pub enum TlsConfig {
    Enabled(RustlsConfig),
    Disabled
}

impl TlsConfig {
    pub async fn load(settings: &Config) -> anyhow::Result<Self> {
        let tls_enabled: bool = settings.get_bool("network.tls.enabled")
            .map_err(|cause| anyhow!("Expected configuration flag 'network.tls.enabled' to be parseable as boolean! {}", cause))?;

        let tls_config = if tls_enabled {
            let cert = {
                let cert_path = project::make_path_absolute(settings.get_string("network.tls.certificate")?)?;
                debug!("Using TLS certificate: {}", cert_path.display());
                assert!(cert_path.exists(), "TLS certificate file at '{}' not found.", cert_path.display());
                fs::read(&cert_path)
                    .context(format!("Error while reading TLS certificate at {}", cert_path.display()))?
            };

            let key = {
                let key_path = project::make_path_absolute(settings.get_string("network.tls.key")?)?;
                debug!("Using TLS key: {}", key_path.display());
                assert!(key_path.exists(), "TLS key file at '{}' not found.", key_path.display());
                fs::read(&key_path)
                    .context(format!("Error while reading TLS key at {}", key_path.display()))?
            };

            let tls_config = RustlsConfig::from_pem(cert, key).await?;

            TlsConfig::Enabled(tls_config)
        } else {
            TlsConfig::Disabled
        };

        Ok(tls_config)
    }
}
