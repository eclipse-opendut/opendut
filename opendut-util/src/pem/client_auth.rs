use anyhow::{anyhow, Context};
use config::Config;
use crate::config::ConfigExt;

use super::{Pem, PemFromConfig};


pub enum ClientAuth {
    Enabled { certs: Vec<Pem>, key: Pem },
    Disabled,
}

impl ClientAuth {
    pub fn load_from_config_for_carl_connection(config: &Config) -> anyhow::Result<Self> {
        Self::load_from_config(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH, None, config)
    }

    pub fn load_from_config(
        config_keys: ClientAuthConfigKeys,
        fallback_config_keys: Option<ClientAuthConfigKeys>,
        config: &Config,
    ) -> anyhow::Result<Self> {

        let is_enabled = match fallback_config_keys {
            Some(fallback_config_keys) => config.get_bool_with_fallback(config_keys.enabled, fallback_config_keys.enabled),
            None => config.get_bool(config_keys.enabled),
        }?;

        if is_enabled {
            let certs =
                Pem::read_from_configured_path_or_content(
                    config_keys.certificate,
                    fallback_config_keys.map(|fallback| fallback.certificate),
                    config
                )
                .context("No client authentication certificate found in configured locations.")?;

            if certs.is_empty() {
                return Err(anyhow!("No certificate found for mTLS client authentication in configured locations."))
            }

            let key =
                Pem::read_from_configured_path_or_content(
                    config_keys.key,
                    fallback_config_keys.map(|fallback| fallback.key),
                    config
                )
                .context("Could not read client authentication key found in configured locations.")?
                .first()
                .ok_or(anyhow!("No key found for mTLS client authentication in configured locations."))?
                .clone();

            Ok(Self::Enabled { certs, key })
        } else {
            Ok(Self::Disabled)
        }
    }
}



#[derive(Clone, Copy)]
pub struct ClientAuthConfigKeys {
    pub prefix: &'static str,
    pub enabled: &'static str,
    pub certificate: &'static str,
    pub key: &'static str,
}
macro_rules! client_auth_config_keys {
    ($prefix:expr) => {
        ClientAuthConfigKeys {
            prefix:      $prefix,
            enabled:     ::const_format::formatcp!("{}.enabled", $prefix),
            certificate: ::const_format::formatcp!("{}.certificate", $prefix),
            key:         ::const_format::formatcp!("{}.key", $prefix),
        }
    };
}
pub(crate) use client_auth_config_keys;
