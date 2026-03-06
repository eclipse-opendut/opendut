use anyhow::Context;
use pem::Pem;
pub use reqwest::Client as ReqwestClient;
use reqwest::Identity;

pub mod oidc {
    use super::{construct_reqwest_identity_from_two_pems, ReqwestClient};
    use crate::pem::{self, ClientAuth, Pem, PemFromConfig};
    use anyhow::Context;
    use config::Config;
    use reqwest::{Certificate, Identity};
    use tracing::debug;


    #[tracing::instrument(name="oidc_client_create", skip_all)]
    pub fn create_from_config(config: &Config) -> anyhow::Result<ReqwestClient> {
        let opendut_ca = Pem::read_from_configured_path_or_content(
            pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CA,
            Some(pem::config_keys::DEFAULT_NETWORK_TLS_CA),
            config
        )?;

        let client_auth = ClientAuth::load_from_config(
            pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH,
            Some(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH),
            config
        )?;

        let identity = match client_auth {
            ClientAuth::Enabled { certs, key } => Some(construct_reqwest_identity_from_two_pems(certs, key)?),
            ClientAuth::Disabled => None,
        };

        build_client(opendut_ca, identity)
    }

    pub fn create_with_ca(ca_certificates: Vec<Pem>) -> anyhow::Result<ReqwestClient> {
        build_client(ca_certificates, None)
    }


    pub(crate) fn build_client(
        ca_certificates: Vec<Pem>,
        client_auth_identity: Option<Identity>,
    ) -> anyhow::Result<ReqwestClient> {

        let mut client = ReqwestClient::builder()
            .redirect(reqwest::redirect::Policy::none());

        debug!("Constructing reqwest client with CA certificates provided.");
        for ca_certificate in ca_certificates  {
            let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes())?;

            client = client.add_root_certificate(reqwest_certificate);
        }

        if let Some(client_auth_identity) = client_auth_identity {
            debug!("Constructing reqwest client with mTLS client auth identity provided.");
            client = client.identity(client_auth_identity);
        }

        client.build()
            .context("Error while building reqwest client")
    }
}

/// `reqwest` does not offer an API to specify two separate PEMs,
/// so we join them by simply putting them underneath each other,
/// which the PEM format allows. See, for example:
/// https://stackoverflow.com/questions/68340665/pem-file-has-two-certificates-what-does-it-mean
/// Certificate order matters: client certificate must be the first in the list or else rustls will raise an error: `keys may not be consistent: KeyMismatch`
fn construct_reqwest_identity_from_two_pems(certificates: Vec<Pem>, key: Pem) -> anyhow::Result<Identity> {
    let client_certificate = certificates.first().cloned()
        .context("No client certificate found when constructing reqwest identity from two PEMs")?;
    let name = crate::pem::read_certificate_subject(&client_certificate)
        .unwrap_or_else(|_| "unknown".to_string());
    tracing::debug!("Constructing reqwest identity for client certificate with subject: {}", name);
    let mut pems: Vec<String> = certificates.into_iter().map(|cert| cert.to_string()).collect();
    pems.push(key.to_string());
    let concatenated_pems = pems.join("\n");

    Identity::from_pem(concatenated_pems.as_bytes())
        .context("Error while constructing reqwest identity from manually joined PEM file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pem::PemFromConfig;

    #[test]
    fn should_construct_reqwest_identity_from_two_pems() -> anyhow::Result<()> {
        use repo_path::repo_path;

        let cert = repo_path!("resources/development/tls/insecure-development-ca.pem");
        let key = repo_path!("resources/development/tls/insecure-development-ca.key");

        let certs = Pem::from_file_path(&cert)?;
        let key = Pem::from_file_path(&key)?.first().cloned().expect("Could not get private key");

        let result = construct_reqwest_identity_from_two_pems(certs, key);

        assert!(result.is_ok());
        Ok(())
    }
}
