use std::net::IpAddr;

use anyhow::anyhow;
use opendut_netbird_client_api::extension::LocalPeerStateExtension;
use opendut_util::settings::LoadedConfig;
use serde::Deserialize;
use tracing::debug;

use crate::common::settings;

#[derive(Debug, Deserialize)]
#[serde(rename_all="kebab-case")]
pub struct VpnConfig {
    pub enabled: bool,
}


pub async fn retrieve_remote_host(settings: &LoadedConfig) -> anyhow::Result<IpAddr> {
    let vpn_config = settings.config.get::<VpnConfig>(settings::key::vpn::table)?;

    let address = if vpn_config.enabled {
        debug!("Determining remote IP address of host in VPN network.");
        let mut client = opendut_netbird_client_api::client::Client::connect().await?;

        let status = client.full_status().await?;

        let host = status.local_peer_state
            .ok_or(anyhow!("NetBird Client did not return a local peer state. May not be logged in. Re-run `edgar setup` to fix this."))?
            .local_ip()?;

        IpAddr::from(host)
    } else {
        settings.config.get::<IpAddr>(settings::key::vpn::disabled::remote::host)
            .map_err(|cause| anyhow!("Configuration value '{field}' must be a valid IP address: {cause}", field=settings::key::vpn::disabled::remote::host))?
    };
    Ok(address)
}
