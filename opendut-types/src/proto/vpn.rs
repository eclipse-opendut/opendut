use crate::proto::{ConversionError, ConversionErrorBuilder};

include!(concat!(env!("OUT_DIR"), "/opendut.types.vpn.rs"));

impl From<crate::vpn::VpnPeerConfig> for VpnPeerConfig {
    fn from(value: crate::vpn::VpnPeerConfig) -> Self {
        match value {
            crate::vpn::VpnPeerConfig::Disabled => {
                VpnPeerConfig {
                    config: Some(vpn_peer_config::Config::Disabled(
                        VpnPeerConfigDisabled {}
                    ))
                }
            }
            crate::vpn::VpnPeerConfig::Netbird { management_url, setup_key } => {
                VpnPeerConfig {
                    config: Some(vpn_peer_config::Config::Netbird(
                        VpnPeerConfigNetbird {
                            management_url: Some(management_url.into()),
                            setup_key: Some(setup_key.into()),
                        }
                    ))
                }
            }
        }
    }
}

impl TryFrom<VpnPeerConfig> for crate::vpn::VpnPeerConfig {
    type Error = ConversionError;

    fn try_from(value: VpnPeerConfig) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<VpnPeerConfig, crate::vpn::VpnPeerConfig>;

        let config = value.config
            .ok_or(ErrorBuilder::new("Config not set"))?;

        let result = match config {
            vpn_peer_config::Config::Disabled(_) => {
                crate::vpn::VpnPeerConfig::Disabled
            }
            vpn_peer_config::Config::Netbird(config) => {
                let VpnPeerConfigNetbird { management_url, setup_key } = config;
                let management_url = management_url
                    .ok_or(ErrorBuilder::new("Management URL not set"))?
                    .try_into()?;
                let setup_key = setup_key
                    .ok_or(ErrorBuilder::new("Setup Key not set"))?
                    .try_into()?;
                crate::vpn::VpnPeerConfig::Netbird {
                    management_url,
                    setup_key,
                }
            },
        };

        Ok(result)
    }
}

impl From<crate::vpn::netbird::SetupKey> for SetupKey {
    fn from(value: crate::vpn::netbird::SetupKey) -> Self {
        Self { uuid: Some(value.uuid.into()) }
    }
}

impl TryFrom<SetupKey> for crate::vpn::netbird::SetupKey {
    type Error = ConversionError;

    fn try_from(value: SetupKey) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<SetupKey, crate::vpn::netbird::SetupKey>;

        let uuid: uuid::Uuid = value.uuid
            .ok_or(ErrorBuilder::new("Setup Key UUID not set"))?
            .into();
        let result = crate::vpn::netbird::SetupKey::from(uuid);
        Ok(result)
    }
}
