mod netbird;
use netbird::NetbirdProcess;

use std::net::IpAddr;
use anyhow::Context;
use opendut_util::settings::LoadedConfig;
use serde::Deserialize;

use crate::common::settings;
use crate::common::settings::netbird::NetbirdClientConfig;

#[must_use]
pub enum VpnProcess {
    Netbird(NetbirdProcess),
    Disabled,
}
impl VpnProcess {
    pub async fn spawn_from_config(settings: &LoadedConfig) -> anyhow::Result<Self> {
        let vpn_config = VpnConfig::load_from_config(settings)?;

        if vpn_config.enabled {
            let config = NetbirdClientConfig::load_from_config(settings)?;
            Self::spawn_as_netbird(config).await
        } else {
            Ok(Self::Disabled)
        }
    }

    pub async fn spawn_as_netbird(config: NetbirdClientConfig) -> anyhow::Result<Self> {
        let netbird = NetbirdProcess::spawn(config).await?;
        Ok(Self::Netbird(netbird))
    }

    pub async fn retrieve_remote_host(&self, settings: &LoadedConfig) -> anyhow::Result<IpAddr> {
        match self {
            VpnProcess::Netbird(netbird) => {
                netbird.retrieve_remote_host().await
            }
            VpnProcess::Disabled => {
                let field = settings::key::vpn::disabled::remote::host;
                let address = settings.get::<IpAddr>(field)
                    .context("Configuration value '{field}' must be a valid IP address")?;
                Ok(address)
            }
        }
    }

    pub async fn terminate(self) -> anyhow::Result<()> {
        match self {
            Self::Netbird(netbird) => netbird.terminate().await,
            Self::Disabled => Ok(()), //do nothing
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
struct VpnConfig {
    pub enabled: bool,
}
impl VpnConfig {
    pub fn load_from_config(settings: &LoadedConfig) -> anyhow::Result<Self> {
        settings.get::<VpnConfig>(settings::key::vpn::table)
            .context("Error while VPN configuration")
    }
}
