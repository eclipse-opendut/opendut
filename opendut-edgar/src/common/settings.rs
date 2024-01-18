use std::path::PathBuf;

use opendut_util::settings::LoadedConfig;

#[allow(non_upper_case_globals)]
pub mod key {
    pub mod peer {
        pub const id: &str = "peer.id";
    }
    pub mod vpn {
        pub const table: &str = "vpn";

        pub mod disabled {
            pub const table: &str = "vpn.disabled";

            pub mod remote {
                pub const host: &str = "vpn.disabled.remote.host";
            }
        }
    }
}

pub fn default_config_file_path() -> PathBuf {
    PathBuf::from("/etc/opendut-network/edgar.toml")
}

pub fn load_with_overrides(overrides: config::Config) -> anyhow::Result<LoadedConfig> {
    let edgar_config_hide_secrets_override = opendut_util::settings::Config::default();

    Ok(opendut_util::settings::load_config("edgar", include_str!("../../edgar.toml"), config::FileFormat::Toml, overrides, edgar_config_hide_secrets_override)?)
}
