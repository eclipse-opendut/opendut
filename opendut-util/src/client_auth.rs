use anyhow::{anyhow, Context};
use config::Config;
use crate::pem::{self, config_keys::ClientAuthConfigKeys, Pem, PemFromConfig};


pub enum ClientAuth {
    Enabled { certs: Vec<Pem>, key: Pem },
    Disabled,
}

impl ClientAuth {
    pub fn load_from_config_for_carl_connection(config: &Config) -> anyhow::Result<Self> {
        Self::load_from_config(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH, None, config)
    }

    pub fn load_from_config(
        config_keys: ClientAuthConfigKeys,
        fallback_config_keys: Option<ClientAuthConfigKeys>,
        config: &Config,
    ) -> anyhow::Result<Self> {

        if config.get_bool(config_keys.enabled)? {

            let certs = Pem::read_from_configured_path_or_content(config_keys.certificate, fallback_config_keys.map(|fallback| fallback.certificate), config)
                .context("No client authentication certificate found in configured locations.")?;

            let key = Pem::read_from_configured_path_or_content(config_keys.key, fallback_config_keys.map(|fallback| fallback.key), config)
                .context("Could not read client authentication key found in configured locations.")?
                .first().cloned().ok_or(anyhow!("No client authentication key found in configured locations."))?;

            Ok(Self::Enabled { certs, key })
        } else {
            Ok(Self::Disabled)
        }
    }
}
