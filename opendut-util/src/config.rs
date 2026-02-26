use config::{Config, ConfigError};
use tracing::trace;

pub trait ConfigExt {
    fn get_bool_with_fallback(&self, config_key: &str, fallback_config_key: &str) -> Result<bool, ConfigError>;
    fn get_optional_bool(&self, config_key: &str) -> Result<Option<bool>, ConfigError>;
}

pub const CONFIG_OPTIONAL_BOOL_UNSET_STRING_VALUE: &str = "unset";

impl ConfigExt for Config {
    /// Retrieves a boolean value from the configuration, with a distinct fallback mechanism for explicit and default values.
    ///
    /// This function attempts to read an *explicit* boolean value associated with `config_key`.
    /// If `config_key` is unset during its retrieval,
    /// it then falls back to reading a *default* boolean value associated with `fallback_config_key`.
    /// This design allows for separate configuration of an explicit override (`config_key`)
    /// and a system-wide default (`fallback_config_key`).
    fn get_bool_with_fallback(&self, config_key: &str, fallback_config_key: &str) -> Result<bool, ConfigError> {
        let config_value = self.get_optional_bool(config_key)?;
        let fallback_value = self.get_bool(fallback_config_key)?;
        match config_value {
            None => {
                trace!("Bool in config key <{config_key}> is unset. Falling back to config key <{fallback_config_key}> with value <{fallback_value}>.");
                Ok(fallback_value)
            }
            Some(value) => {
                trace!("Read bool from config key <{config_key}> with value <{value}>.");
                Ok(value)
            }
        }
    }

    /// Retrieves an optional boolean value from the configuration.
    ///
    /// This function attempts to read a string value associated with `config_key`.
    /// If the retrieved string value matches `unset`,
    /// it indicates that the boolean is explicitly unset, and the function returns `Ok(None)`.
    /// Otherwise, it attempts to parse the string as a boolean. If successful, it returns `Ok(Some(bool))`.
    fn get_optional_bool(&self, config_key: &str) -> Result<Option<bool>, ConfigError> {
        let value = self.get_string(config_key)?;
        if value == CONFIG_OPTIONAL_BOOL_UNSET_STRING_VALUE {
            Ok(None)
        } else {
            Ok(Some(self.get_bool(config_key)?))
        }
    }
}

#[cfg(test)]
mod tests {
    mod get_bool_with_fallback {
        use crate::config::{ConfigExt, CONFIG_OPTIONAL_BOOL_UNSET_STRING_VALUE};
        #[test]
        fn should_be_enabled_when_explicitly_enabled() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, true)?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled, false)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled)?;
            assert!(is_enabled, "Expected value to be enabled when explicitly enabled.");
            Ok(())
        }

        #[test]
        fn should_be_disabled_when_explicitly_disabled() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, false)?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled, true)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled)?;
            assert!(!is_enabled, "Expected value be disable when explicitly disabled.");
            Ok(())
        }

        #[test]
        fn should_use_default_true_when_explicit_config_cannot_be_loaded() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, CONFIG_OPTIONAL_BOOL_UNSET_STRING_VALUE)?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled, true)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled)?;
            assert!(is_enabled, "Expected value be enabled when falling back to default.");
            Ok(())
        }

        #[test]
        fn should_use_default_false_when_explicit_config_cannot_be_loaded() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, CONFIG_OPTIONAL_BOOL_UNSET_STRING_VALUE)?
                .set_override(crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled, false)?
                .build()?;
            let is_enabled = config.get_bool_with_fallback(crate::pem::config_keys::OPENTELEMETRY_TLS_CLIENT_AUTH.enabled, crate::pem::config_keys::DEFAULT_NETWORK_TLS_CLIENT_AUTH.enabled)?;
            assert!(!is_enabled, "Expected value be enabled when falling back to default.");
            Ok(())
        }
    }

    mod get_optional_bool {
        use crate::config::{ConfigExt, CONFIG_OPTIONAL_BOOL_UNSET_STRING_VALUE};
        const CONFIG_KEY: &str = "test";

        #[test]
        fn boolean_config_value_can_be_read_as_string() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(CONFIG_KEY, false)?
                .build()?;
            let result = config.get_string(CONFIG_KEY);
            assert!(result.is_ok(), "Expected value to be readable as string.");
            assert_eq!("false", result?);
            Ok(())
        }

        #[test]
        fn unset_value_is_none() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(CONFIG_KEY, CONFIG_OPTIONAL_BOOL_UNSET_STRING_VALUE)?
                .build()?;
            let result = config.get_optional_bool(CONFIG_KEY)?;
            assert!(result.is_none(), "Expected unset boolean value to be none.");
            Ok(())
        }

        #[test]
        fn given_value_is_true() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(CONFIG_KEY, true)?
                .build()?;
            let result = config.get_optional_bool(CONFIG_KEY)?;
            assert!(result.is_some(), "Expected boolean value to be some.");
            assert_eq!(result, Some(true), "Expected boolean value to be set.");
            Ok(())
        }

        #[test]
        fn given_value_is_false() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(CONFIG_KEY, false)?
                .build()?;
            let result = config.get_optional_bool(CONFIG_KEY)?;
            assert!(result.is_some(), "Expected boolean value to be some.");
            assert_eq!(result, Some(false), "Expected boolean value to be set.");
            Ok(())
        }

        #[test]
        fn should_return_error_when_config_value_contains_nonsense() -> anyhow::Result<()> {
            let config = config::Config::builder()
                .set_override(CONFIG_KEY, "nonsense")?
                .build()?;
            let result = config.get_optional_bool(CONFIG_KEY);
            assert!(result.is_err(), "Expected error when config value contains something else than unset value.");
            Ok(())
        }
    }
}
