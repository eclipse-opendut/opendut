use crate::proto::{conversion, ConversionError, ConversionErrorBuilder, ConversionResult};

include!(concat!(env!("OUT_DIR"), "/opendut.types.vpn.rs"));


conversion! {
    type Model = crate::vpn::VpnPeerConfiguration;
    type Proto = VpnPeerConfig;

    fn from(value: Model) -> Proto {
        match value {
            crate::vpn::VpnPeerConfiguration::Disabled => {
                VpnPeerConfig {
                    config: Some(vpn_peer_config::Config::Disabled(
                        VpnPeerConfigDisabled {}
                    ))
                }
            }
            crate::vpn::VpnPeerConfiguration::Netbird { management_url, setup_key } => {
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

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let config = extract!(value.config)?;

        let result = match config {
            vpn_peer_config::Config::Disabled(_) => {
                crate::vpn::VpnPeerConfiguration::Disabled
            }
            vpn_peer_config::Config::Netbird(config) => {
                let VpnPeerConfigNetbird { management_url, setup_key } = config;
                let management_url = extract!(management_url)?.try_into()?;

                let setup_key = extract!(setup_key)?.try_into()?;

                crate::vpn::VpnPeerConfiguration::Netbird {
                    management_url,
                    setup_key,
                }
            },
        };

        Ok(result)
    }
}

conversion! {
    type Model = crate::vpn::netbird::SetupKey;
    type Proto = SetupKey;

    fn from(value: Model) -> Proto {
        Proto { uuid: Some(value.uuid.into()) }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let uuid: uuid::Uuid = extract!(value.uuid)?.into();

        let result = crate::vpn::netbird::SetupKey::from(uuid);

        Ok(result)
    }
}
