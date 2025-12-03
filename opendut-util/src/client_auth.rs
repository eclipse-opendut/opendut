use anyhow::Context;
use config::Config;
use crate::pem::{self, Pem, PemFromConfig};


pub enum ClientAuth {
    Enabled { cert: Pem, key: Pem },
    Disabled,
}

impl ClientAuth {
    pub fn load_from_config(config: &Config) -> anyhow::Result<Self> {

        if config.get_bool("network.tls.client.auth.enabled")? {

            let cert = Pem::read_from_config_keys_with_env_fallback(&[pem::config_keys::NETWORK_TLS_CLIENT_AUTH_CERT], config)?
                .context("No client authentication certificate found in configured locations.")?;

            let key = Pem::read_from_config_keys_with_env_fallback(&[pem::config_keys::NETWORK_TLS_CLIENT_AUTH_KEY], config)?
                .context("No client authentication key found in configured locations.")?;

            Ok(Self::Enabled { cert, key })
        } else {
            Ok(Self::Disabled)
        }
    }
}
