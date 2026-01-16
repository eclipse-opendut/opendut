use config::Config;
use tracing::trace;

pub trait ConfigExt {
    fn get_bool_with_fallback(&self, config_key: &str, fallback_config_key: &str) -> bool;
}

impl ConfigExt for Config {
    /// Retrieves a boolean value from the configuration, with a distinct fallback mechanism for explicit and default values.
    ///
    /// This function attempts to read an *explicit* boolean value associated with `config_key`.
    /// If `config_key` is not found or an error occurs during its retrieval,
    /// it then falls back to reading a *default* boolean value associated with `fallback_config_key`.
    /// If `fallback_config_key` is also not found or yields an error, the function ultimately defaults to `false`.
    /// This design allows for separate configuration of an explicit override (`config_key`)
    /// and a system-wide default (`fallback_config_key`).
    fn get_bool_with_fallback(&self, config_key: &str, fallback_config_key: &str) -> bool {
        let fallback_value = self.get_bool(fallback_config_key).unwrap_or(false);
        let config_value = self.get_bool(config_key);
        match config_value {
            Ok(value) => {
                trace!("Read bool from config key <{config_key}> with value <{value}>.");
                value
            }
            Err(config_error) => {
                trace!("Could not read bool from config key <{config_key}>, due to error: <{config_error}>. Falling back to config key <{fallback_config_key}> with value <{fallback_value}>.");
                fallback_value
            }
        }
    }
}

#[cfg(test)]
mod tests {
    mod get_bool_with_fallback {
        use crate::config::ConfigExt;
        #[test]
        fn should_be_enabled_when_explicitly_enabled() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, true)?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, false)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED);
            assert!(is_enabled, "Expected value to be enabled when explicitly enabled.");
            Ok(())
        }

        #[test]
        fn should_be_disabled_when_explicitly_disabled() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, false)?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, true)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED);
            assert!(!is_enabled, "Expected value be disable when explicitly disabled.");
            Ok(())
        }

        #[test]
        fn should_use_default_true_when_explicit_config_cannot_be_loaded() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, "use default defined in network.tls.client.auth, otherwise set this to 'true' or 'false'")?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, true)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED);
            assert!(is_enabled, "Expected value be enabled when falling back to default.");
            Ok(())
        }

        #[test]
        fn should_use_default_false_when_explicit_config_cannot_be_loaded() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, "use default defined in network.tls.client.auth, otherwise set this to 'true' or 'false'")?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED, false)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH_ENABLED, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH_ENABLED);
            assert!(!is_enabled, "Expected value be enabled when falling back to default.");
            Ok(())
        }
    }

}