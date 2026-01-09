use anyhow::Context;
use axum_server::tls_rustls::RustlsConfig;
use config::Config;
use std::sync::Arc;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::pki_types::pem::PemObject;
use rustls::RootCertStore;
use rustls::server::WebPkiClientVerifier;
use tracing::{debug, info};
use opendut_util::pem::{Pem, PemFromConfig};
use opendut_util::pem;

pub enum TlsConfig {
    Enabled(RustlsConfig),
    Disabled
}

impl TlsConfig {
    /// Create TLS configuration for GRPC server from application settings.
    /// See https://github.com/hyperium/tonic/blob/master/examples/src/tls_rustls/server.rs
    pub async fn load(settings: &Config) -> anyhow::Result<Self> {
        let tls_enabled: bool = settings.get_bool("network.tls.enabled")
            .context("Expected configuration flag 'network.tls.enabled' to be parseable as boolean!")?;

        let tls_config = if tls_enabled {
            let server_certs = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_CERTIFICATE, None, settings)
                .context("No TLS certificate found in configured locations.")?
                .into_iter()
                .map(|pem| CertificateDer::from_pem_slice(pem.to_string().as_bytes()))
                .collect::<Result<Vec<CertificateDer>, rustls::pki_types::pem::Error>>()?;

            let server_key = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_KEY, None, settings)
                .context("No TLS private key found in configured locations.")?
                .into_iter()
                .next()
                .map(|pem| PrivateKeyDer::from_pem_slice(pem.to_string().as_bytes()))
                .ok_or_else(|| anyhow::anyhow!("No TLS private key found in configured locations."))?
                .context("Could not parse TLS private key.")?;

            let config_builder = rustls::ServerConfig::builder();

            let mutual_tls_enabled: bool = settings.get_bool("network.tls.server.auth.enabled")
                .context("Expected configuration flag 'network.tls.server.auth' to be parseable as boolean!")?;

            let config = if mutual_tls_enabled {
                info!("Mutual TLS authentication is enabled. Clients connecting to the server are required to present a client certificate.");

                let trusted_client_cas = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_SERVER_AUTH_CA, Some(pem::config_keys::DEFAULT_NETWORK_TLS_CA), settings)
                    .context("No trusted client certificate authorities found in configured locations.")?
                    .into_iter()
                    .map(|pem| CertificateDer::from_pem_slice(pem.to_string().as_bytes()))
                    .collect::<Result<Vec<CertificateDer>, rustls::pki_types::pem::Error>>()?;
                let mut roots = RootCertStore::empty();
                roots.add_parsable_certificates(trusted_client_cas);
                let client_cert_verifier = WebPkiClientVerifier::builder(Arc::new(roots)).build().expect("Failed to build WebPkiClientVerifier");
                config_builder.with_client_cert_verifier(client_cert_verifier)
            } else {
                debug!("Mutual TLS authentication is disabled.");
                config_builder.with_no_client_auth()
            };
            let mut config = config
                .with_single_cert(server_certs, server_key)
                .expect("Failed to create rustls ServerConfig with provided certificate and key.");
            // Enable HTTP/2 Application-Layer Protocol Negotiation (ALPN) or else gRPC clients will not be able to connect (transport error).
            config.alpn_protocols = vec![b"h2".to_vec()];

            let tls_config = RustlsConfig::from_config(Arc::new(config));
            TlsConfig::Enabled(tls_config)
        } else {
            TlsConfig::Disabled
        };

        Ok(tls_config)
    }
}
