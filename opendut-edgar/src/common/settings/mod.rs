pub mod netbird;

use std::path::PathBuf;

use opendut_util::settings::LoadedConfig;

pub const CONFIG_APPLICATION_PREFIX: &str = "edgar";

#[allow(non_upper_case_globals)]
pub mod key {
    pub mod peer {
        pub const id: &str = "peer.id";
    }
    pub mod vpn {
        pub const table: &str = "vpn";

        pub mod disabled {
            pub mod remote {
                pub const host: &str = "vpn.disabled.remote.host";
            }
        }
    }
    pub mod netbird {
        pub mod client {
            pub mod log {
                pub const level: &str = "vpn.netbird.client.log.level";
            }
        }
    }
}

pub fn default_config_file_path() -> PathBuf {
    PathBuf::from("/etc/opendut/edgar.toml")
}

pub fn netbird_config_file_path() -> PathBuf {
    PathBuf::from("/etc/opendut/edgar/netbird/config.json")
}

pub fn load_with_overrides(overrides: config::Config) -> anyhow::Result<LoadedConfig> {
    let edgar_config_hide_secrets_override = opendut_util::settings::Config::default();

    let loaded_config = opendut_util::settings::load_config(
        CONFIG_APPLICATION_PREFIX,
        include_str!("../../../edgar.toml"),
        config::FileFormat::Toml,
        overrides,
        edgar_config_hide_secrets_override
    )?;

    Ok(loaded_config)
}
