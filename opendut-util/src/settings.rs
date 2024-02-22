use std::fmt::Debug;
use std::path::PathBuf;

pub use config::{Config, ConfigError, FileFormat};
use home::home_dir;

use crate::project;

#[derive(thiserror::Error, Debug)]
pub enum LoadError {
    #[error("Failed to load config: {0}")]
    ConfigError(#[from] ConfigError)
}

#[derive(Clone)]
pub struct LoadedConfig {
    pub config: Config,
    pub redacted_config: Config,
    pub config_files_used: Vec<PathBuf>,
    pub config_files_declared: Vec<PathBuf>,
}

impl Debug for LoadedConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedConfig")
            .field("config", &self.redacted_config)
            .field("config_files_used", &self.config_files_used)
            .field("config_files_declared", &self.config_files_declared)
            .finish()
    }
}

/// Load configuration from files and environment variables used by openDuT.
///
/// This includes in following order:
/// * A default configuration, provided as a string
/// * A development configuration, read from the crate's directory
/// * A system configuration, read from `/etc/opendut/{name}.toml`
/// * A user configuration, read from `[XDG_CONFIG_HOME|~/.config]/opendut/{name}/config.toml`
/// * Environment variables prefixed with `OPENDUT_{NAME}_`
/// * The `overrides` passed as parameter.
///
pub fn load_config(name: &str, defaults: &str, defaults_format: FileFormat, overrides: Config, secret_redacted_overrides: Config) -> Result<LoadedConfig, LoadError> {

    let development_config = format!("opendut-{name}/{name}-development.toml");
    let system_config = format!("/etc/opendut/{name}.toml");
    let user_config = format!("opendut/{name}/config.toml");

    let builder = Config::builder()
        .add_source(config::File::from_str(defaults, defaults_format));

    let mut config_files = Vec::new();

    if project::is_running_in_development() {
        config_files.push(project::make_path_absolute(development_config).ok())
    }

    config_files.push(Some(PathBuf::from(system_config)));

    match std::env::var("XDG_CONFIG_HOME") {
        Ok(xdg_config_home) => {
            config_files.push(Some(PathBuf::from(xdg_config_home).join(user_config)));
        }
        Err(_) => {
            config_files.push(home_dir().map(|path| path.join(".config").join(user_config)));
        }
    }

    let (sources_used, sources_declared): (Vec<PathBuf>, Vec<PathBuf>) = config_files.into_iter()
        .fold((Vec::new(), Vec::new()), |(mut used, mut declared), path| {
            if let Some(path) = path {
                declared.push(Clone::clone(&path));
                if path.exists() && path.is_file() {
                    used.push(path);
                }
            }
            (used, declared)
        });

    let builder = sources_used.iter()
        .cloned()
        .fold(builder, |builder, path| {
            builder.add_source(config::File::from(path).required(false))
        });

    let builder = builder.add_source(
        config::Environment::with_prefix(&format!("OPENDUT_{}", name.to_uppercase()))
            .separator("_")
            .try_parsing(true)
    );

    let settings = builder.add_source(overrides);
    let secret_redacted_settings = settings.clone()
        .add_source(secret_redacted_overrides);

    Ok(LoadedConfig {
        config: settings.build()?,
        redacted_config: secret_redacted_settings.build()?,
        config_files_used: sources_used,
        config_files_declared: sources_declared,
    })
}
