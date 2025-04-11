use crate::proto::{conversion, ConversionError, ConversionErrorBuilder, ConversionResult};
use crate::proto::vpn::VpnPeerConfig;

use super::util::{NetworkInterfaceDescriptor, NetworkInterfaceName};

pub mod configuration;
pub mod executor;

include!(concat!(env!("OUT_DIR"), "/opendut.types.peer.rs"));

conversion! {
    type Model = crate::peer::PeerId;
    type Proto = PeerId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.uuid.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| Model { uuid: uuid.into() })
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

conversion! {
    type Model = crate::peer::PeerName;
    type Proto = PeerName;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.0
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}

conversion! {
    type Model = crate::peer::PeerLocation;
    type Proto = PeerLocation;

    fn from(value: Model) -> Proto {
        Proto {
            value: value.0
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        Model::try_from(value.value)
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
    }
}
impl From<&str> for PeerLocation {
    fn from(value: &str) -> Self {
        Self {
            value: String::from(value)
        }
    }
}

conversion! {
    type Model = crate::peer::PeerNetworkDescriptor;
    type Proto = PeerNetworkDescriptor;

    fn from(value: Model) -> Proto {
        Proto {
            interfaces: value
                .interfaces
                .into_iter()
                .map(NetworkInterfaceDescriptor::from)
                .collect(),
            bridge_name: value.bridge_name.map(NetworkInterfaceName::from),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
         let bridge_name =  value.bridge_name
             .map(crate::util::net::NetworkInterfaceName::try_from)
             .transpose()?;
        value
            .interfaces
            .into_iter()
            .map(NetworkInterfaceDescriptor::try_into)
            .collect::<Result<_, _>>()
            .map(|interfaces| Model { interfaces, bridge_name})
    }
}

conversion! {
    type Model = crate::peer::PeerDescriptor;
    type Proto = PeerDescriptor;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            location: Some(value.location.unwrap_or_default().into()),
            network: Some(value.network.into()),
            topology: Some(value.topology.into()),
            executors: Some(value.executors.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id = extract!(value.id)?
            .try_into()?;

        let name = extract!(value.name)?
            .try_into()?;

        let location = value.location
            .map(crate::peer::PeerLocation::try_from)
            .transpose()?;

        let network = extract!(value.network)?
            .try_into()?;

        let topology = extract!(value.topology)?
            .try_into()?;

        let executors = extract!(value.executors)?
            .try_into()?;
        
        Ok(Model {
            id,
            name,
            location,
            network,
            topology,
            executors,
        })
    }
}

conversion! {
    type Model = crate::peer::PeerSetup;
    type Proto = PeerSetup;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            carl: Some(value.carl.into()),
            ca: Some(value.ca.into()),
            vpn: Some(value.vpn.into()),
            auth_config: Some(value.auth_config.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let id: crate::peer::PeerId = extract!(value.id)?
            .try_into()?;

        let carl: url::Url = extract!(value.carl)
            .and_then(|url|
                url::Url::parse(&url.value)
                    .map_err(|cause| ErrorBuilder::message(format!("Carl URL could not be parsed: {}", cause)))
            )?;

        let ca: crate::util::net::Certificate = extract!(value.ca)
            .and_then(crate::util::net::Certificate::try_from)?;

        let vpn: crate::vpn::VpnPeerConfiguration = extract!(value.vpn)
            .and_then(VpnPeerConfig::try_into)?;

        let auth_config = extract!(value.auth_config)?
            .try_into()?;

        Ok(Model {
            id, carl, ca, auth_config, vpn
        })
    }
}

conversion! {
    type Model = crate::peer::state::PeerState;
    type Proto = PeerState;

    fn from(state: Model) -> Proto {
        let connection = match state.connection {
            crate::peer::state::PeerConnectionState::Offline => {
                PeerConnectionState {
                    state: Some(peer_connection_state::State::Offline(PeerOffline {})),
                }
            }
            crate::peer::state::PeerConnectionState::Online { remote_host } => {
                PeerConnectionState {
                    state: Some(peer_connection_state::State::Online(PeerOnline {
                        remote_host: Some(remote_host.into()),
                    }))
                }
            }
        };
        let member = match state.member {
            crate::peer::state::PeerMemberState::Available => {
                PeerMemberState {
                    state: Some(peer_member_state::State::Available(PeerMemberAvailable {})),
                }
            }
            crate::peer::state::PeerMemberState::Blocked { by_cluster } => {
                PeerMemberState {
                    state: Some(peer_member_state::State::Blocked(PeerMemberBlocked {
                        by_cluster: Some(by_cluster.into()),
                    })),
                }
            }
        };
        PeerState {
            connection: Some(connection),
            member: Some(member),
        }
    }

    fn try_from(state: Proto) -> ConversionResult<Model> {
        let proto_member = extract!(state.member)?;
        let proto_connection = extract!(state.connection)?;

        let connection = crate::peer::state::PeerConnectionState::try_from(proto_connection)?;
        let member = crate::peer::state::PeerMemberState::try_from(proto_member)?;

        Ok(crate::peer::state::PeerState {
            connection,
            member,
        })
    }
}

conversion! {
    type Model = crate::peer::state::PeerConnectionState;
    type Proto = PeerConnectionState;

    fn from(state: Model) -> Proto {
        let state = match state {
            Model::Offline => {
                peer_connection_state::State::Offline(PeerOffline {})
            }
            Model::Online { remote_host  } => {
                peer_connection_state::State::Online(PeerOnline { remote_host: Some(remote_host.into()) })
            }
        };
        PeerConnectionState {
            state: Some(state)
        }
    }
    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let proto_state = extract!(proto.state)?;
        let state = match proto_state {
            peer_connection_state::State::Online(online) => {
                let remote_host: std::net::IpAddr = extract!(online.remote_host)?
                    .try_into()?;
                Model::Online { remote_host }
            }
            peer_connection_state::State::Offline(_) => {
                Model::Offline {}
            }
        };
        Ok(state)
    }
}

conversion! {
    type Model = crate::peer::state::PeerMemberState;
    type Proto = PeerMemberState;

    fn from(state: Model) -> Proto {
        let state = match state {
            Model::Available => {
                peer_member_state::State::Available(PeerMemberAvailable {})
            }
            Model::Blocked { by_cluster  } => {
                peer_member_state::State::Blocked(PeerMemberBlocked { by_cluster: Some(by_cluster.into()) })
            }
        };
        PeerMemberState {
            state: Some(state),
        }
    }
    fn try_from(proto: Proto) -> ConversionResult<Model> {
        let proto_member = extract!(proto.state)?;
        let member = match proto_member {
            peer_member_state::State::Available(_) => {
                crate::peer::state::PeerMemberState::Available {}
            }
            peer_member_state::State::Blocked(blocked) => {
                crate::peer::state::PeerMemberState::Blocked {
                    by_cluster: extract!(blocked.by_cluster)?
                        .try_into()?
                }
            }
        };
        Ok(member)
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
            ok(eq(&native))
        );

        assert_that!(&PeerId::from(native), eq(&proto));

        Ok(())
    }

    #[test]
    fn A_PeerLocation_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

        let peer_location = "Ulm";

        let native = crate::peer::PeerLocation::try_from(peer_location)?;
        let proto = PeerLocation::from(peer_location);

        assert_that!(
            crate::peer::PeerLocation::try_from(Clone::clone(&proto)),
            ok(eq(&native))
        );

        assert_that!(PeerLocation::from(native), eq(&proto));

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
            let native = crate::peer::state::PeerState::default();
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(&native))
            );
        }

        { // Up/Available
            let native = crate::peer::state::PeerState {
                connection: crate::peer::state::PeerConnectionState::Online {
                    remote_host: native_remote_host
                },
                member: crate::peer::state::PeerMemberState::Available,
            };
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(&native))
            );
        }

        { // Up/Blocked/Deploying
            let native = crate::peer::state::PeerState {
                connection: crate::peer::state::PeerConnectionState::Online {
                    remote_host: native_remote_host
                },
                member: crate::peer::state::PeerMemberState::Blocked {
                    by_cluster: crate::cluster::ClusterId::random()
                },
            };
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(&native))
            );
        }

        Ok(())
    }
}
