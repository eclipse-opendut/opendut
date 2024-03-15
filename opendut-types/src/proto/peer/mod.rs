use crate::proto;
use crate::proto::{ConversionError, ConversionErrorBuilder};
use crate::proto::vpn::VpnPeerConfig;

use super::util::NetworkInterfaceDescriptor;

pub mod configuration;
pub mod executor;

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.rs"));

impl From<crate::peer::PeerId> for PeerId {
    fn from(value: crate::peer::PeerId) -> Self {
        Self {
            uuid: Some(value.0.into())
        }
    }
}

impl TryFrom<PeerId> for crate::peer::PeerId {
    type Error = ConversionError;

    fn try_from(value: PeerId) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerId, crate::peer::PeerId>;

        value.uuid
            .ok_or(ErrorBuilder::new("Uuid not set"))
            .map(|uuid| Self(uuid.into()))
    }
}

impl From<uuid::Uuid> for PeerId {
    fn from(value: uuid::Uuid) -> Self {
        let (msb, lsb) = value.as_u64_pair();
        Self {
            uuid: Some(crate::proto::util::Uuid { msb, lsb })
        }
    }
}

impl From<crate::peer::PeerName> for PeerName {
    fn from(value: crate::peer::PeerName) -> Self {
        Self {
            value: value.0
        }
    }
}

impl TryFrom<PeerName> for crate::peer::PeerName {
    type Error = ConversionError;

    fn try_from(value: PeerName) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerName, crate::peer::PeerName>;

        crate::peer::PeerName::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::PeerLocation> for PeerLocation {
    fn from(value: crate::peer::PeerLocation) -> Self {
        Self {
            value: value.0
        }
    }
}

impl From<&str> for PeerLocation {
    fn from(value: &str) -> Self {
        Self {
            value: String::from(value)
        }
    }
}

impl TryFrom<PeerLocation> for crate::peer::PeerLocation {
    type Error = ConversionError;

    fn try_from(value: PeerLocation) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerLocation, crate::peer::PeerLocation>;

        crate::peer::PeerLocation::try_from(value.value)
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

impl From<crate::peer::PeerNetworkConfiguration> for PeerNetworkConfiguration {
    fn from(value: crate::peer::PeerNetworkConfiguration) -> Self {
        Self {
            interfaces: value
                .interfaces
                .into_iter()
                .map(NetworkInterfaceDescriptor::from)
                .collect(),
        }
    }
}

impl TryFrom<PeerNetworkConfiguration> for crate::peer::PeerNetworkConfiguration {
    type Error = ConversionError;

    fn try_from(value: PeerNetworkConfiguration) -> Result<Self, Self::Error> {
        value
            .interfaces
            .into_iter()
            .map(NetworkInterfaceDescriptor::try_into)
            .collect::<Result<_, _>>()
            .map(|interfaces| Self { interfaces })
    }
}

impl From<crate::peer::PeerDescriptor> for PeerDescriptor {
    fn from(value: crate::peer::PeerDescriptor) -> Self {
        Self {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            location: Some(value.location.unwrap_or_default().into()),
            network_configuration: Some(value.network_configuration.into()),
            topology: Some(value.topology.into()),
            executors: Some(value.executors.into()),
        }
    }
}

impl TryFrom<PeerDescriptor> for crate::peer::PeerDescriptor {
    type Error = ConversionError;

    fn try_from(value: PeerDescriptor) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerDescriptor, crate::peer::PeerDescriptor>;

        let id = value.id
            .ok_or(ErrorBuilder::new("Id not set"))?
            .try_into()?;

        let name = value.name
            .ok_or(ErrorBuilder::new("Name not set"))?
            .try_into()?;

        let location = value.location
            .map(crate::peer::PeerLocation::try_from)
            .transpose()?;

        let network_configuration = value.network_configuration
            .ok_or(ErrorBuilder::new("Network configuration not set"))?
            .try_into()?;

        let topology = value.topology
            .ok_or(ErrorBuilder::new("Topology not set"))?
            .try_into()?;

        let executors = value.executors
            .ok_or(ErrorBuilder::new("Executor not set"))?
            .try_into()?;
        
        Ok(crate::peer::PeerDescriptor {
            id,
            name,
            location,
            network_configuration,
            topology,
            executors,
        })
    }
}

impl From<crate::peer::PeerSetup> for PeerSetup {
    fn from(value: crate::peer::PeerSetup) -> Self {
        Self {
            id: Some(value.id.into()),
            carl: Some(value.carl.into()),
            ca: Some(value.ca.into()),
            vpn: Some(value.vpn.into()),
            auth_config: Some(value.auth_config.into()),
        }
    }
}

impl TryFrom<PeerSetup> for crate::peer::PeerSetup {
    type Error = ConversionError;

    fn try_from(value: PeerSetup) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerSetup, crate::peer::PeerSetup>;

        let id: crate::peer::PeerId = value.id
            .ok_or(ErrorBuilder::new("PeerId not set"))?
            .try_into()?;

        let carl: url::Url = value.carl
            .ok_or(ErrorBuilder::new("Carl not set"))
            .and_then(|url| url::Url::parse(&url.value)
                .map_err(|cause| ErrorBuilder::new(format!("Carl URL could not be parsed: {}", cause))))?;

        let ca: crate::util::net::Certificate = value.ca
            .ok_or(ErrorBuilder::new("No CA Certificate provided."))
            .and_then(crate::util::net::Certificate::try_from)?;

        let vpn: crate::vpn::VpnPeerConfiguration = value.vpn
            .ok_or(ErrorBuilder::new("VpnConfig not set"))
            .and_then(VpnPeerConfig::try_into)?;

        let auth_config = value.auth_config
            .ok_or(ErrorBuilder::new("PeerId not set"))?
            .try_into()?;

        Ok(Self {
            id,
            carl,
            ca,
            auth_config,
            vpn,
        })
    }
}

impl From<crate::peer::state::PeerState> for PeerState {
    fn from(state: crate::peer::state::PeerState) -> Self {
        match state {
            crate::peer::state::PeerState::Down => {
                PeerState {
                    inner: Some(peer_state::Inner::Down(PeerStateDown {}))
                }
            },
            crate::peer::state::PeerState::Up { inner, remote_host } => {
                let remote_host: proto::util::IpAddress = remote_host.into();
                let remote_host = Some(remote_host);


                match inner {
                    crate::peer::state::PeerUpState::Available => {
                        PeerState {
                            inner: Some(peer_state::Inner::Up(PeerStateUp {
                                inner: Some(peer_state_up::Inner::Available(PeerStateUpAvailable {})),
                                remote_host,
                            }))
                        }
                    },
                    crate::peer::state::PeerUpState::Blocked(inner) => {
                        match inner {
                            crate::peer::state::PeerBlockedState::Deploying => {
                                PeerState {
                                    inner: Some(peer_state::Inner::Up(PeerStateUp {
                                        inner: Some(peer_state_up::Inner::Blocked(PeerStateUpBlocked {
                                            inner: Some(peer_state_up_blocked::Inner::Deploying(PeerStateUpBlockedDeploying {}))
                                        })),
                                        remote_host,
                                    }))
                                }
                            },
                            crate::peer::state::PeerBlockedState::Member => {
                                PeerState {
                                    inner: Some(peer_state::Inner::Up(PeerStateUp {
                                        inner: Some(peer_state_up::Inner::Blocked(PeerStateUpBlocked {
                                            inner: Some(peer_state_up_blocked::Inner::Member(PeerStateUpBlockedMember {}))
                                        })),
                                        remote_host,
                                    }))
                                }
                            },
                            crate::peer::state::PeerBlockedState::Undeploying => {
                                PeerState {
                                    inner: Some(peer_state::Inner::Up(PeerStateUp {
                                        inner: Some(peer_state_up::Inner::Blocked(PeerStateUpBlocked {
                                            inner: Some(peer_state_up_blocked::Inner::Undeploying(PeerStateUpBlockedUndeploying {}))
                                        })),
                                        remote_host,
                                    }))
                                }
                            },
                        }
                    },
                }
            }
        }
    }
}


impl TryFrom<PeerState> for crate::peer::state::PeerState {
    type Error = ConversionError;

    fn try_from(state: PeerState) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerState, crate::peer::state::PeerState>;

        let inner = state.inner
            .ok_or(ErrorBuilder::new("Inner state not set"))?;

        match inner {
            peer_state::Inner::Down(_) => {
                Ok(crate::peer::state::PeerState::Down)
            }
            peer_state::Inner::Up(PeerStateUp { inner, remote_host }) => {

                let remote_host: std::net::IpAddr = remote_host
                    .ok_or(ErrorBuilder::new("field 'remote_host' not set"))?
                    .try_into()?;


                let inner = inner
                    .ok_or(ErrorBuilder::new("Inner 'Up' state not set"))?;

                match inner {
                    peer_state_up::Inner::Available(_) => {
                        Ok(crate::peer::state::PeerState::Up {
                            inner: crate::peer::state::PeerUpState::Available,
                            remote_host,
                        })
                    }
                    peer_state_up::Inner::Blocked(PeerStateUpBlocked { inner }) => {

                        let inner = inner
                            .ok_or(ErrorBuilder::new("Inner 'Blocked' state not set"))?;

                        match inner {
                            peer_state_up_blocked::Inner::Deploying(_) => {
                                Ok(crate::peer::state::PeerState::Up {
                                    inner: crate::peer::state::PeerUpState::Blocked(
                                        crate::peer::state::PeerBlockedState::Deploying
                                    ),
                                    remote_host,
                                })
                            }
                            peer_state_up_blocked::Inner::Member(_) => {
                                Ok(crate::peer::state::PeerState::Up {
                                    inner: crate::peer::state::PeerUpState::Blocked(
                                        crate::peer::state::PeerBlockedState::Member
                                    ),
                                    remote_host,
                                })
                            }
                            peer_state_up_blocked::Inner::Undeploying(_) => {
                                Ok(crate::peer::state::PeerState::Up {
                                    inner: crate::peer::state::PeerUpState::Blocked(
                                        crate::peer::state::PeerBlockedState::Undeploying
                                    ),
                                    remote_host,
                                })
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use std::net::IpAddr;
    use std::str::FromStr;

    use googletest::prelude::*;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn A_PeerId_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

        let peer_id = Uuid::new_v4();

        let native = crate::peer::PeerId::from(peer_id);
        let proto = PeerId::from(peer_id);

        assert_that!(
            crate::peer::PeerId::try_from(Clone::clone(&proto)),
            ok(eq(native))
        );

        assert_that!(PeerId::from(native), eq(proto));

        Ok(())
    }

    #[test]
    fn A_PeerLocation_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

        let peer_location = "Ulm";

        let native = crate::peer::PeerLocation::try_from(peer_location).unwrap();
        let proto = PeerLocation::from(peer_location);

        assert_that!(
            crate::peer::PeerLocation::try_from(Clone::clone(&proto)),
            ok(eq(native.clone()))
        );

        assert_that!(PeerLocation::from(native), eq(proto));

        Ok(())
    }

    #[test]
    fn A_invalid_PeerLocation_should_not_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

        let peer_location_with_invalid_start_char = "-Ulm";
        let peer_location_with_invalid_characters = "Ul/&$#@m";
        let peer_location_is_empty = "";

        assert!(crate::peer::PeerLocation::try_from(peer_location_with_invalid_start_char).is_err());
        assert!(crate::peer::PeerLocation::try_from(peer_location_with_invalid_characters).is_err());
        assert!(crate::peer::PeerLocation::try_from(peer_location_is_empty).is_ok());

        Ok(())
    }

    #[test]
    fn A_PeerState_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

        let native_remote_host = IpAddr::from_str("1.2.3.4")?;

        { // Down
            let native = crate::peer::state::PeerState::Down;
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Available
            let native = crate::peer::state::PeerState::Up {
                inner: crate::peer::state::PeerUpState::Available,
                remote_host: native_remote_host,
            };
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Blocked/Deploying
            let native = crate::peer::state::PeerState::Up {
                inner: crate::peer::state::PeerUpState::Blocked(
                    crate::peer::state::PeerBlockedState::Deploying
                ),
                remote_host: native_remote_host,
            };
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Blocked/Member
            let native = crate::peer::state::PeerState::Up {
                inner: crate::peer::state::PeerUpState::Blocked(
                    crate::peer::state::PeerBlockedState::Member
                ),
                remote_host: native_remote_host,
            };
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Blocked/Undeploying
            let native = crate::peer::state::PeerState::Up {
                inner: crate::peer::state::PeerUpState::Blocked(
                    crate::peer::state::PeerBlockedState::Undeploying
                ),
                remote_host: native_remote_host,
            };
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        Ok(())
    }
}
