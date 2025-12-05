use anyhow::Context;
use config::Config;
use crate::pem::{self, Pem, PemFromConfig};


pub enum ClientAuth {
    Enabled { cert: Pem, key: Pem },
    Disabled,
}

impl ClientAuth {
    pub fn load_from_config(config: &Config) -> anyhow::Result<Self> {

        if config.get_bool(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED)? {

            let cert = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_CERTIFICATE, None, config)?
                .context("No client authentication certificate found in configured locations.")?;

            let key = Pem::read_from_configured_path_or_content(pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_KEY, None, config)?
                .context("No client authentication key found in configured locations.")?;

            Ok(Self::Enabled { cert, key })
        } else {
            Ok(Self::Disabled)
        }
    }
}
