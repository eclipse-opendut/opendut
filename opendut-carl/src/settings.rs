use opendut_util::settings::{LoadedConfig, LoadError};

pub fn load_with_overrides(overrides: config::Config) -> Result<LoadedConfig, LoadError> {
    let carl_config_hide_secrets_override = config::Config::builder()
        .set_override("vpn.netbird.auth.secret", "redacted")?
        .set_override("network.oidc.client.secret", "redacted")?
        .build()?;

    opendut_util::settings::load_config("carl", include_str!("../carl.toml"), config::FileFormat::Toml, overrides, carl_config_hide_secrets_override)
}

#[cfg(test)]
pub fn load_defaults() -> Result<LoadedConfig, LoadError> {
    load_with_overrides(config::Config::default())
}
