use anyhow::{anyhow, Context};
use config::Config;
use crate::pem::{self, Pem, PemFromConfig};


pub enum ClientAuth {
    Enabled { certs: Vec<Pem>, key: Pem },
    Disabled,
}

impl ClientAuth {
    pub fn load_from_config(config: &Config) -> anyhow::Result<Self> {

        if config.get_bool(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED)? {

            let certs = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE, None, config)
                .context("No client authentication certificate found in configured locations.")?;

            let key = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_KEY, None, config)
                .context("Could not read client authentication key found in configured locations.")?
                .first().cloned().ok_or(anyhow!("No client authentication key found in configured locations."))?;

            Ok(Self::Enabled { certs, key })
        } else {
            Ok(Self::Disabled)
        }
    }
}
