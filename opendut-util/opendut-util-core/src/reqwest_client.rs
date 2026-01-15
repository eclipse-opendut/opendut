use anyhow::Context;
use pem::Pem;
pub use reqwest::Client as ReqwestClient;
use reqwest::Identity;

pub mod oidc {
    use super::{construct_reqwest_identity_from_two_pems, ReqwestClient};
    use crate::pem::{self, Pem, PemFromConfig};
    use anyhow::{anyhow, Context};
    use config::Config;
    use reqwest::{Certificate, Identity};
    use tracing::{debug, trace};

    /// Determines whether OIDC client authentication is enabled,
    /// based on the specific OIDC setting or falling back to the default setting.
    pub(crate) fn oidc_client_auth_enabled(config: &Config) -> bool {
        let default_client_auth_enabled = config.get_bool(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED).unwrap_or(false);
        let oidc_client_auth_enabled_result = config.get_bool(pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_ENABLED);
        oidc_client_auth_enabled_result.unwrap_or_else(|config_error| {
            trace!("Could not read OIDC client auth enabled setting due to invalid value, config error: {}. Falling back to default client auth enabled setting: {}", config_error, default_client_auth_enabled);
            default_client_auth_enabled
        })
    }

    #[tracing::instrument(name="oidc_client_create", skip_all)]
    pub fn create_from_config(config: &Config) -> anyhow::Result<ReqwestClient> {
        let opendut_ca = Pem::read_from_configured_path_or_content(
            pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CA,
            Some(pem::config_keys::DEFAULT_NETWORK_TLS_CA),
            config
        )?;

        let identity =
            if oidc_client_auth_enabled(config) {
                let certificates = Pem::read_from_configured_path_or_content(
                    pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_CERTIFICATE,
                    Some(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE),
                    config
                )?;
                if certificates.is_empty() {
                    return Err(anyhow!("No certificate found for mTLS client authentication in OIDC"))
                }

                let key = Pem::read_from_configured_path_or_content(
                    pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_KEY,
                    Some(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_KEY),
                    config
                )?.first().cloned();
                match key {
                    None => {
                        return Err(anyhow!("No key found for mTLS client authentication in OIDC"))
                    }
                    Some(key) => {
                        Some(construct_reqwest_identity_from_two_pems(certificates, key)?)
                    }
                }
            } else {
                None
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
            .redirect(reqwest::redirect::Policy::none())
            .tls_built_in_root_certs(true);

        for ca_certificate in ca_certificates  {
            debug!("Constructing reqwest client with CA certificate provided.");
            let reqwest_certificate = Certificate::from_pem(ca_certificate.to_string().as_bytes())
                .map_err(|cause| anyhow!(cause.to_string()))?;

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

    #[test]
    fn should_enable_client_auth_when_oidc_client_auth_is_explicitly_enabled() -> anyhow::Result<()> {
        let config = config::Config::builder()
            .set_override(crate::pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_ENABLED, true)?
            .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, false)?
            .build()?;
        let is_enabled = oidc::oidc_client_auth_enabled(&config);
        assert!(is_enabled, "Expected client auth to be enabled when OIDC client auth is explicitly enabled.");
        Ok(())
    }

    #[test]
    fn should_disable_client_auth_when_oidc_client_auth_is_explicitly_disabled() -> anyhow::Result<()> {
        let config = config::Config::builder()
            .set_override(crate::pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_ENABLED, false)?
            .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, true)?
            .build()?;
        let is_enabled = oidc::oidc_client_auth_enabled(&config);
        assert!(!is_enabled, "Expected client auth to be disabled when OIDC client auth is explicitly disabled.");
        Ok(())
    }

    #[test]
    fn should_enable_client_auth_when_oidc_client_auth_is_explicitly_enabled_and_default_value_contains_nonsense() -> anyhow::Result<()> {
        let config = config::Config::builder()
            .set_override(crate::pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_ENABLED, true)?
            .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, "nonsense")?
            .build()?;
        let is_enabled = oidc::oidc_client_auth_enabled(&config);
        assert!(is_enabled, "Expected client auth to be enabled when OIDC client auth is explicitly enabled.");
        Ok(())
    }

    #[test]
    fn should_use_default_true_client_auth_when_oidc_client_auth_is_set_to_a_non_boolean_value() -> anyhow::Result<()> {
        let config = config::Config::builder()
            .set_override(crate::pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_ENABLED, "use default defined in network.tls.client.auth, otherwise set this to 'true' or 'false'")?
            .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, true)?
            .build()?;
        let is_enabled = oidc::oidc_client_auth_enabled(&config);
        assert!(is_enabled);
        Ok(())
    }

    #[test]
    fn should_use_default_false_client_auth_when_oidc_client_auth_is_set_to_a_non_boolean_value() -> anyhow::Result<()> {
        let config = config::Config::builder()
            .set_override(crate::pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_ENABLED, "use default defined in network.tls.client.auth, otherwise set this to 'true' or 'false'")?
            .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, false)?
            .build()?;
        let is_enabled = oidc::oidc_client_auth_enabled(&config);
        assert!(!is_enabled);
        Ok(())
    }

    #[test]
    fn should_evaluate_to_false_when_config_contains_nonsense_values() -> anyhow::Result<()> {
        let config = config::Config::builder()
            .set_override(crate::pem::config_keys::NETWORK_OIDC_CLIENT_TLS_CLIENT_AUTH_ENABLED, "nonsense")?
            .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, "nonsense")?
            .build()?;
        let is_enabled = oidc::oidc_client_auth_enabled(&config);
        assert!(!is_enabled);
        Ok(())
    }


}
