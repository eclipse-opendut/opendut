use anyhow::anyhow;
use config::Config;
use pem::Pem;
use reqwest::Certificate;
use crate::pem::PemFromConfig;

#[derive(Debug, Clone)]
pub struct OidcReqwestClient {}
pub use reqwest::{Client as ReqwestClient};


impl OidcReqwestClient {
    pub fn from_config(config: &Config) -> anyhow::Result<ReqwestClient> {
        let opendut_ca = Pem::read_from_config(config)?;
        Self::build_client(opendut_ca)
    }

    fn build_client(ca_certificate: Option<Pem>) -> anyhow::Result<ReqwestClient> {
        let mut client = ReqwestClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .tls_built_in_root_certs(true);
        if let Some(ca_certificate) = ca_certificate {
            let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes().iter().as_slice())
                .map_err(|cause| anyhow!(cause.to_string()))?;
            client = client.add_root_certificate(reqwest_certificate);
        }
        client.build()
            .map_err(|cause| anyhow!(cause.to_string()))
    }

    pub fn from_pem(ca_certificate: Pem) -> anyhow::Result<ReqwestClient> {
        OidcReqwestClient::build_client(Some(ca_certificate))
    }
}
