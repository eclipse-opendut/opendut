use crate::proto::{ConversionError, ConversionErrorBuilder};
use crate::proto::vpn::VpnPeerConfig;

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

impl From<crate::peer::PeerDescriptor> for PeerDescriptor {
    fn from(value: crate::peer::PeerDescriptor) -> Self {
        Self {
            id: Some(value.id.into()),
            name: Some(value.name.into()),
            topology: Some(value.topology.into()),
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

        let topology = value.topology
            .ok_or(ErrorBuilder::new("Topology not set"))?
            .try_into()?;

        Ok(crate::peer::PeerDescriptor {
            id,
            name,
            topology,
        })
    }
}

impl From<crate::peer::PeerSetup> for PeerSetup {
    fn from(value: crate::peer::PeerSetup) -> Self {
        Self {
            id: Some(value.id.into()),
            carl:  Some(value.carl.into()),
            vpn: Some(value.vpn.into()),
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

        let vpn: crate::vpn::VpnPeerConfig = value.vpn
            .ok_or(ErrorBuilder::new("VpnConfig not set"))
            .and_then(VpnPeerConfig::try_into)?;

        Ok(Self {
            id,
            carl,
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
            crate::peer::state::PeerState::Up(inner) => {
                match inner {
                    crate::peer::state::PeerUpState::Available => {
                        PeerState {
                            inner: Some(peer_state::Inner::Up(PeerStateUp {
                                inner: Some(peer_state_up::Inner::Available(PeerStateUpAvailable {}))
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
                                        }))
                                    }))
                                }
                            },
                            crate::peer::state::PeerBlockedState::Member => {
                                PeerState {
                                    inner: Some(peer_state::Inner::Up(PeerStateUp {
                                        inner: Some(peer_state_up::Inner::Blocked(PeerStateUpBlocked {
                                            inner: Some(peer_state_up_blocked::Inner::Member(PeerStateUpBlockedMember {}))
                                        }))
                                    }))
                                }
                            },
                            crate::peer::state::PeerBlockedState::Undeploying => {
                                PeerState {
                                    inner: Some(peer_state::Inner::Up(PeerStateUp {
                                        inner: Some(peer_state_up::Inner::Blocked(PeerStateUpBlocked {
                                            inner: Some(peer_state_up_blocked::Inner::Undeploying(PeerStateUpBlockedUndeploying {}))
                                        }))
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
            peer_state::Inner::Up(PeerStateUp { inner }) => {

                let inner = inner
                    .ok_or(ErrorBuilder::new("Inner 'Up' state not set"))?;

                match inner {
                    peer_state_up::Inner::Available(_) => {
                        Ok(crate::peer::state::PeerState::Up(
                            crate::peer::state::PeerUpState::Available))
                    }
                    peer_state_up::Inner::Blocked(PeerStateUpBlocked { inner }) => {

                        let inner = inner
                            .ok_or(ErrorBuilder::new("Inner 'Blocked' state not set"))?;

                        match inner {
                            peer_state_up_blocked::Inner::Deploying(_) => {
                                Ok(crate::peer::state::PeerState::Up(
                                    crate::peer::state::PeerUpState::Blocked(
                                        crate::peer::state::PeerBlockedState::Deploying)))
                            }
                            peer_state_up_blocked::Inner::Member(_) => {
                                Ok(crate::peer::state::PeerState::Up(
                                    crate::peer::state::PeerUpState::Blocked(
                                        crate::peer::state::PeerBlockedState::Member)))
                            }
                            peer_state_up_blocked::Inner::Undeploying(_) => {
                                Ok(crate::peer::state::PeerState::Up(
                                    crate::peer::state::PeerUpState::Blocked(
                                        crate::peer::state::PeerBlockedState::Undeploying)))
                            }
                        }
                    }
                }
            }
        }
    }
}

impl From<crate::peer::state::PeerStates> for Vec<PeerState> {
    fn from(value: crate::peer::state::PeerStates) -> Self {
        value.0.into_iter().map(PeerState::from).collect::<Vec<_>>()
    }
}

impl TryFrom<Vec<PeerState>> for crate::peer::state::PeerStates {
    type Error = ConversionError;

    fn try_from(value: Vec<PeerState>) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<Vec<PeerState>, crate::peer::state::PeerStates>;

        value.into_iter().map(crate::peer::state::PeerState::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(|states| crate::peer::state::PeerStates(states))
            .map_err(|cause| ErrorBuilder::new(cause.to_string()))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
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
    fn A_PeerState_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

        { // Down
            let native = crate::peer::state::PeerState::Down;
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Available
            let native = crate::peer::state::PeerState::Up(
                crate::peer::state::PeerUpState::Available
            );
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Blocked/Deploying
            let native = crate::peer::state::PeerState::Up(
                crate::peer::state::PeerUpState::Blocked(
                    crate::peer::state::PeerBlockedState::Deploying
                )
            );
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Blocked/Member
            let native = crate::peer::state::PeerState::Up(
                crate::peer::state::PeerUpState::Blocked(
                    crate::peer::state::PeerBlockedState::Member
                )
            );
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        { // Up/Blocked/Undeploying
            let native = crate::peer::state::PeerState::Up(
                crate::peer::state::PeerUpState::Blocked(
                    crate::peer::state::PeerBlockedState::Undeploying
                )
            );
            let proto: PeerState = Clone::clone(&native).into();

            assert_that!(
                crate::peer::state::PeerState::try_from(Clone::clone(&proto)),
                ok(eq(native))
            );
        }

        Ok(())
    }
}
