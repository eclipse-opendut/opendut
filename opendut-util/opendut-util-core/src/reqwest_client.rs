
pub use reqwest::{Client as ReqwestClient};

pub mod oidc {
    use anyhow::anyhow;
    use config::Config;
    use reqwest::Certificate;
    use crate::pem::{self, Pem, PemFromConfig};
    use super::ReqwestClient;

    pub fn create_from_config(config: &Config) -> anyhow::Result<ReqwestClient> {
        let opendut_ca = Pem::read_from_configured_path_or_content(
            pem::config_keys::OIDC_CLIENT_CA,
            Some(pem::config_keys::DEFAULT_NETWORK_TLS_CA),
            config
        )?;

        build_client(opendut_ca)
    }

    pub fn create_with_ca(ca_certificate: Pem) -> anyhow::Result<ReqwestClient> {
        build_client(Some(ca_certificate))
    }


    fn build_client(ca_certificate: Option<Pem>) -> anyhow::Result<ReqwestClient> {

        let mut client = ReqwestClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .tls_built_in_root_certs(true);

        if let Some(ca_certificate) = ca_certificate {
            let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes())
                .map_err(|cause| anyhow!(cause.to_string()))?;

            client = client.add_root_certificate(reqwest_certificate);
        }

        client.build()
            .map_err(|cause| anyhow!(cause.to_string()))
    }
}
