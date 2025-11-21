use anyhow::Context;
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
            .context("Expected configuration flag 'network.tls.enabled' to be parseable as boolean!")?;

        let tls_config = if tls_enabled {
            let cert = {
                let cert_path = project::make_path_absolute(settings.get_string("network.tls.certificate")?)?;
                debug!("Using TLS certificate: {cert_path:?}");

                assert!(cert_path.exists(), "TLS certificate file at {cert_path:?} not found.");

                fs::read(&cert_path)
                    .context(format!("Error while reading TLS certificate at {cert_path:?}"))?
            };

            let key = {
                let key_path = project::make_path_absolute(settings.get_string("network.tls.key")?)?;
                debug!("Using TLS key: {key_path:?}");

                assert!(key_path.exists(), "TLS key file at {key_path:?} not found.");

                fs::read(&key_path)
                    .context(format!("Error while reading TLS key at {key_path:?}"))?
            };

            let tls_config = RustlsConfig::from_pem(cert, key).await?;

            TlsConfig::Enabled(tls_config)
        } else {
            TlsConfig::Disabled
        };

        Ok(tls_config)
    }
}
