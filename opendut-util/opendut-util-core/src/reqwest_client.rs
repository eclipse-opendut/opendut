use anyhow::Context;
use pem::Pem;
pub use reqwest::{Client as ReqwestClient};
use reqwest::Identity;

pub mod oidc {
    use anyhow::{anyhow, Context};
    use config::Config;
    use reqwest::{Certificate, Identity};
    use crate::pem::{self, Pem, PemFromConfig};
    use super::{construct_reqwest_identity_from_two_pems, ReqwestClient};

    pub fn create_from_config(config: &Config) -> anyhow::Result<ReqwestClient> {
        let opendut_ca = Pem::read_from_configured_path_or_content(
            pem::config_keys::OIDC_TLS_CA,
            Some(pem::config_keys::DEFAULT_NETWORK_TLS_CA),
            config
        )?;

        let identity =
            if config.get_bool(pem::config_keys::OIDC_TLS_CLIENT_AUTH_ENABLED)? {
                let certificate = Pem::read_from_configured_path_or_content(
                    pem::config_keys::OIDC_TLS_CLIENT_AUTH_CERTIFICATE,
                    Some(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE),
                    config
                )?.context("No certificate found for mTLS client authentication in OIDC")?;

                let key = Pem::read_from_configured_path_or_content(
                    pem::config_keys::OIDC_TLS_CLIENT_AUTH_KEY,
                    Some(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_KEY),
                    config
                )?.context("No key found for mTLS client authentication in OIDC")?;

                let identity = construct_reqwest_identity_from_two_pems(certificate, key)?;

                Some(identity)
            } else {
                None
            };

        build_client(opendut_ca, identity)
    }

    pub fn create_with_ca(ca_certificate: Pem) -> anyhow::Result<ReqwestClient> {
        build_client(Some(ca_certificate), None)
    }


    fn build_client(
        ca_certificate: Option<Pem>,
        client_auth_identity: Option<Identity>,
    ) -> anyhow::Result<ReqwestClient> {

        let mut client = ReqwestClient::builder()
            .redirect(reqwest::redirect::Policy::none())
            .tls_built_in_root_certs(true);

        if let Some(ca_certificate) = ca_certificate {
            let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes())
                .map_err(|cause| anyhow!(cause.to_string()))?;

            client = client.add_root_certificate(reqwest_certificate);
        }

        if let Some(client_auth_identity) = client_auth_identity {
            client = client.identity(client_auth_identity);
        }

        client.build()
            .map_err(|cause| anyhow!(cause.to_string()))
    }
}

/// `reqwest` does not offer an API to specify two separate PEMs,
/// so we join them by simply putting them underneath each other,
/// which the PEM format allows. See, for example:
/// https://stackoverflow.com/questions/68340665/pem-file-has-two-certificates-what-does-it-mean
fn construct_reqwest_identity_from_two_pems(certificate: Pem, key: Pem) -> anyhow::Result<Identity> {
    let pem = [certificate.to_string(), key.to_string()].join("\n");

    Identity::from_pem(pem.as_bytes())
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

        let cert = Pem::from_file_path(&cert)?;
        let key = Pem::from_file_path(&key)?;

        let result = construct_reqwest_identity_from_two_pems(cert, key);

        assert!(result.is_ok());
        Ok(())
    }
}
