use std::marker::PhantomData;

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("Could not convert from `{from}` to `{to}`: {details}")]
pub struct ConversionError {
    from: &'static str,
    to: &'static str,
    details: String,
}

impl ConversionError {
    pub fn new<From, To>(details: impl Into<String>) -> Self {
        Self {
            from: std::any::type_name::<From>(),
            to: std::any::type_name::<To>(),
            details: details.into(),
        }
    }
}

pub struct ConversionErrorBuilder<From, To> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
}

#[allow(clippy::new_ret_no_self)]
impl<From, To> ConversionErrorBuilder<From, To> {
    pub fn new(details: impl Into<String>) -> ConversionError {
        ConversionError::new::<From, To>(details)
    }
}

pub mod cluster {
    use crate::proto::{ConversionError, ConversionErrorBuilder};
    use crate::proto::topology::DeviceId;

    include!(concat!(env!("OUT_DIR"), "/opendut.types.cluster.rs"));

    impl From<crate::cluster::ClusterId> for ClusterId {
        fn from(value: crate::cluster::ClusterId) -> Self {
            Self {
                uuid: Some(value.0.into())
            }
        }
    }

    impl TryFrom<ClusterId> for crate::cluster::ClusterId {
        type Error = ConversionError;

        fn try_from(value: ClusterId) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<ClusterId, crate::cluster::ClusterId>;

            value.uuid
                .ok_or(ErrorBuilder::new("Uuid not set"))
                .map(|uuid| Self(uuid.into()))
        }
    }

    impl From<uuid::Uuid> for ClusterId {
        fn from(value: uuid::Uuid) -> Self {
            Self {
                uuid: Some(value.into())
            }
        }
    }

    impl From<crate::cluster::ClusterName> for ClusterName {
        fn from(value: crate::cluster::ClusterName) -> Self {
            Self {
                value: value.0
            }
        }
    }

    impl TryFrom<ClusterName> for crate::cluster::ClusterName {
        type Error = ConversionError;

        fn try_from(value: ClusterName) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<ClusterName, crate::cluster::ClusterName>;

            crate::cluster::ClusterName::try_from(value.value)
                .map_err(|cause| ErrorBuilder::new(cause.to_string()))
        }
    }

    impl From<crate::cluster::ClusterConfiguration> for ClusterConfiguration {
        fn from(configuration: crate::cluster::ClusterConfiguration) -> Self {
            Self {
                id: Some(configuration.id.into()),
                name: Some(configuration.name.into()),
                leader: Some(configuration.leader.into()),
                devices: configuration.devices.into_iter()
                            .map(DeviceId::from)
                            .collect(),
            }
        }
    }

    impl TryFrom<ClusterConfiguration> for crate::cluster::ClusterConfiguration {
        type Error = ConversionError;

        fn try_from(configuration: ClusterConfiguration) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<ClusterConfiguration, crate::cluster::ClusterConfiguration>;

            let cluster_id: crate::cluster::ClusterId = configuration.id
                .ok_or(ErrorBuilder::new("Id not set"))?
                .try_into()?;

            let cluster_name: crate::cluster::ClusterName = configuration.name
                .ok_or(ErrorBuilder::new("Name not set"))?
                .try_into()?;

            let leader: crate::peer::PeerId = configuration.leader
                .ok_or(ErrorBuilder::new("Leader not set"))?
                .try_into()?;

            Ok(Self {
                id: cluster_id,
                name: cluster_name,
                leader,
                devices: configuration.devices.into_iter()
                            .map(DeviceId::try_into)
                            .collect::<Result<_, _>>()?,
            })
        }
    }

    impl From<crate::cluster::ClusterDeployment> for ClusterDeployment {
        fn from(deployment: crate::cluster::ClusterDeployment) -> Self {
            Self {
                id: Some(deployment.id.into()),
            }
        }
    }

    impl TryFrom<ClusterDeployment> for crate::cluster::ClusterDeployment {
        type Error = ConversionError;

        fn try_from(deployment: ClusterDeployment) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<ClusterDeployment, crate::cluster::ClusterDeployment>;

            let cluster_id: crate::cluster::ClusterId = deployment.id
                .ok_or(ErrorBuilder::new("Id not set"))?
                .try_into()?;

            Ok(Self {
                id: cluster_id,
            })
        }
    }

    impl From<crate::cluster::state::ClusterState> for ClusterState {
        fn from(state: crate::cluster::state::ClusterState) -> Self {
            match state {
                crate::cluster::state::ClusterState::Undeployed => {
                    ClusterState {
                        inner: Some(cluster_state::Inner::Undeployed(ClusterStateUndeployed {}))
                    }
                },
                crate::cluster::state::ClusterState::Deploying => {
                    ClusterState {
                        inner: Some(cluster_state::Inner::Deploying(ClusterStateDeploying {}))
                    }
                },
                crate::cluster::state::ClusterState::Deployed(inner) => {
                    match inner {
                        crate::cluster::state::DeployedClusterState::Unhealthy => {
                            ClusterState {
                                inner: Some(cluster_state::Inner::Deployed(ClusterStateDeployed {
                                    inner: Some(cluster_state_deployed::Inner::Unhealthy(ClusterStateDeployedUnhealthy {}))
                                }))
                            }
                        },
                        crate::cluster::state::DeployedClusterState::Healthy => {
                            ClusterState {
                                inner: Some(cluster_state::Inner::Deployed(ClusterStateDeployed {
                                    inner: Some(cluster_state_deployed::Inner::Healthy(ClusterStateDeployedHealthy {}))
                                }))
                            }
                        },
                    }
                }
            }
        }
    }

    impl TryFrom<ClusterState> for crate::cluster::state::ClusterState {
        type Error = ConversionError;

        fn try_from(state: ClusterState) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<ClusterState, crate::cluster::state::ClusterState>;

            let inner = state.inner
                .ok_or(ErrorBuilder::new("Inner state not set"))?;

            match inner {
                cluster_state::Inner::Undeployed(_) => {
                    Ok(crate::cluster::state::ClusterState::Undeployed)
                }
                cluster_state::Inner::Deploying(_) => {
                    Ok(crate::cluster::state::ClusterState::Deploying)
                }
                cluster_state::Inner::Deployed(state) => {
                    let inner = state.inner
                        .ok_or(ErrorBuilder::new("Inner state not set"))?;
                    let inner = match inner {
                        cluster_state_deployed::Inner::Unhealthy(_) => {
                            crate::cluster::state::DeployedClusterState::Unhealthy
                        }
                        cluster_state_deployed::Inner::Healthy(_) => {
                            crate::cluster::state::DeployedClusterState::Healthy
                        }
                    };
                    Ok(crate::cluster::state::ClusterState::Deployed(inner))
                }
            }
        }
    }

    impl From<crate::cluster::state::ClusterStates> for Vec<ClusterState> {
        fn from(value: crate::cluster::state::ClusterStates) -> Self {
            value.0.into_iter().map(ClusterState::from).collect::<Vec<_>>()
        }
    }

    impl TryFrom<Vec<ClusterState>> for crate::cluster::state::ClusterStates {
        type Error = ConversionError;

        fn try_from(value: Vec<ClusterState>) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Vec<ClusterState>, crate::cluster::state::ClusterStates>;

            value.into_iter().map(crate::cluster::state::ClusterState::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map(|states| crate::cluster::state::ClusterStates(states))
                .map_err(|cause| ErrorBuilder::new(cause.to_string()))
        }
    }

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod test {
        use googletest::prelude::*;

        use super::*;

        #[test]
        fn A_ClusterState_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

            { // Undeployed
                let native = crate::cluster::state::ClusterState::Undeployed;
                let proto: ClusterState = Clone::clone(&native).into();

                assert_that!(
                    crate::cluster::state::ClusterState::try_from(Clone::clone(&proto)),
                    ok(eq(native))
                );
            }

            { // Deploying
                let native = crate::cluster::state::ClusterState::Deploying;
                let proto: ClusterState = Clone::clone(&native).into();

                assert_that!(
                    crate::cluster::state::ClusterState::try_from(Clone::clone(&proto)),
                    ok(eq(native))
                );
            }

            { // Deployed/Unhealthy
                let native = crate::cluster::state::ClusterState::Deployed(
                    crate::cluster::state::DeployedClusterState::Unhealthy
                );
                let proto: ClusterState = Clone::clone(&native).into();

                assert_that!(
                    crate::cluster::state::ClusterState::try_from(Clone::clone(&proto)),
                    ok(eq(native))
                );
            }

            { // Deployed/Healthy
                let native = crate::cluster::state::ClusterState::Deployed(
                    crate::cluster::state::DeployedClusterState::Healthy
                );
                let proto: ClusterState = Clone::clone(&native).into();

                assert_that!(
                    crate::cluster::state::ClusterState::try_from(Clone::clone(&proto)),
                    ok(eq(native))
                );
            }

            Ok(())
        }
    }
}

pub mod peer {
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
}

pub mod topology {
    use crate::proto::{ConversionError, ConversionErrorBuilder};

    include!(concat!(env!("OUT_DIR"), "/opendut.types.topology.rs"));

    impl From<crate::topology::DeviceId> for DeviceId {
        fn from(value: crate::topology::DeviceId) -> Self {
            Self {
                uuid: Some(value.0.into())
            }
        }
    }

    impl TryFrom<DeviceId> for crate::topology::DeviceId {
        type Error = ConversionError;

        fn try_from(value: DeviceId) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<DeviceId, crate::topology::DeviceId>;

            value.uuid
                .ok_or(ErrorBuilder::new("Uuid not set"))
                .map(|uuid| Self(uuid.into()))
        }
    }

    impl From<uuid::Uuid> for DeviceId {
        fn from(value: uuid::Uuid) -> Self {
            Self {
                uuid: Some(value.into())
            }
        }
    }

    impl From<crate::topology::Topology> for Topology {
        fn from(value: crate::topology::Topology) -> Self {
            Self {
                devices: value.devices
                    .into_iter()
                    .map(Device::from)
                    .collect(),
            }
        }
    }

    impl TryFrom<Topology> for crate::topology::Topology {
        type Error = ConversionError;

        fn try_from(value: Topology) -> Result<Self, Self::Error> {
            value.devices
                .into_iter()
                .map(Device::try_into)
                .collect::<Result<_, _>>()
                .map(|devices| Self {
                    devices
                })
        }
    }

    impl From<crate::topology::Device> for Device {
        fn from(value: crate::topology::Device) -> Self {
            Self {
                id: Some(value.id.into()),
                name: value.name,
                description: value.description,
                location: value.location,
                interface: Some(value.interface.into()),
                tags: value.tags,
            }
        }
    }

    impl TryFrom<Device> for crate::topology::Device {
        type Error = ConversionError;

        fn try_from(value: Device) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Device, crate::topology::Device>;

            let device_id: crate::topology::DeviceId = value.id
                .ok_or(ErrorBuilder::new("Id not set"))?
                .try_into()?;
            let interface: crate::topology::InterfaceName = value.interface
                .ok_or(ErrorBuilder::new("Interface not set"))?
                .try_into()?;

            Ok(Self {
                id: device_id,
                name: value.name,
                description: value.description,
                location: value.location,
                interface,
                tags: value.tags,
            })
        }
    }

    impl From<crate::topology::InterfaceName> for InterfaceName {
        fn from(value: crate::topology::InterfaceName) -> Self {
            Self {
                name: value.name(),
            }
        }
    }
    impl TryFrom<InterfaceName> for crate::topology::InterfaceName {
        type Error = ConversionError;

        fn try_from(value: InterfaceName) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Device, crate::topology::Device>;

            crate::topology::InterfaceName::try_from(value.name)
                .map_err(|cause| ErrorBuilder::new(format!("Failed to parse InterfaceName from proto: {cause}")))
        }
    }

    #[cfg(test)]
    #[allow(non_snake_case)]
    mod tests {
        use googletest::prelude::*;
        use uuid::Uuid;

        use super::*;

        #[test]
        fn A_Topology_should_be_convertable_to_its_proto_and_vice_versa() -> Result<()> {

            let device_id_1 = Uuid::new_v4();
            let device_id_2 = Uuid::new_v4();

            let native = crate::topology::Topology {
                devices: vec![
                    crate::topology::Device {
                        id: Clone::clone(&device_id_1).into(),
                        name: String::from("device-1"),
                        description: String::from("Some device"),
                        location: String::from("Ulm"),
                        interface: crate::topology::InterfaceName::try_from("tap0").unwrap(),
                        tags: vec![String::from("tag-1"), String::from("tag-2")],
                    },
                    crate::topology::Device {
                        id: Clone::clone(&device_id_2).into(),
                        name: String::from("device-2"),
                        description: String::from("Some other device"),
                        location: String::from("Stuttgart"),
                        interface: crate::topology::InterfaceName::try_from("tap1").unwrap(),
                        tags: vec![String::from("tag-2")],
                    }
                ],
            };

            let proto = Topology {
                devices: vec![
                    Device {
                        id: Some(Clone::clone(&device_id_1).into()),
                        name: String::from("device-1"),
                        description: String::from("Some device"),
                        location: String::from("Ulm"),
                        interface: Some(InterfaceName { name: String::from("tap0") }),
                        tags: vec![String::from("tag-1"), String::from("tag-2")],
                    },
                    Device {
                        id: Some(Clone::clone(&device_id_2).into()),
                        name: String::from("device-2"),
                        description: String::from("Some other device"),
                        location: String::from("Stuttgart"),
                        interface: Some(InterfaceName { name: String::from("tap1") }),
                        tags: vec![String::from("tag-2")],
                    }
                ],
            };

            verify_that!(native, eq(crate::topology::Topology::try_from(Clone::clone(&proto))?))?;

            verify_that!(proto, eq(Topology::try_from(native)?))?;

            Ok(())
        }

        #[test]
        fn Converting_a_proto_Topology_to_a_native_Topology_should_fail_if_the_id_of_a_device_is_not_set() -> Result<()> {

            let proto = Topology {
                devices: vec![
                    Device {
                        id: None,
                        name: String::from("device-1"),
                        description: String::from("Some device"),
                        location: String::from("Ulm"),
                        interface: Some(InterfaceName { name: String::from("tap0") }),
                        tags: vec![String::from("tag-1"), String::from("tag-2")],
                    },
                ],
            };

            let result = crate::topology::Topology::try_from(proto);

            verify_that!(result, err(eq(ConversionError::new::<Device, crate::topology::Device>("Id not set"))))?;

            Ok(())
        }
    }
}

pub mod util {
    use crate::proto::{ConversionError, ConversionErrorBuilder};
    use crate::util;

    include!(concat!(env!("OUT_DIR"), "/opendut.types.util.rs"));

    impl From<uuid::Uuid> for Uuid {
        fn from(value: uuid::Uuid) -> Self {
            let (msb, lsb) = value.as_u64_pair();
            Self { msb, lsb }
        }
    }

    impl From<Uuid> for uuid::Uuid {
        fn from(value: Uuid) -> Self {
            Self::from_u64_pair(value.msb, value.lsb)
        }
    }

    impl From<String> for Hostname {
        fn from(value: String) -> Self {
            Self { value }
        }
    }

    impl From<Hostname> for String {
        fn from(value: Hostname) -> Self {
            value.value
        }
    }

    impl From<util::Hostname> for Hostname {
        fn from(value: util::Hostname) -> Self {
            Self { value: value.0 }
        }
    }

    impl From<Hostname> for util::Hostname {
        fn from(value: Hostname) -> Self {
            util::Hostname(value.value)
        }
    }

    impl From<u16> for Port {
        fn from(value: u16) -> Self {
            Self { value: value as u32 }
        }
    }

    impl TryFrom<Port> for u16 {
        type Error = ConversionError;

        fn try_from(value: Port) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Port, u16>;

            value.value
                .try_into()
                .map_err(|_| ErrorBuilder::new("Port value is out of range"))
        }
    }

    impl From<util::Port> for Port {
        fn from(value: util::Port) -> Self {
            Self { value: value.0 as u32 }
        }
    }

    impl TryFrom<Port> for util::Port {
        type Error = ConversionError;

        fn try_from(value: Port) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Port, u16>;

            let port: u16 = value.value
                .try_into()
                .map_err(|_| ErrorBuilder::new("Port value is out of range"))?;

            Ok(util::Port(port))
        }
    }

    impl From<url::Url> for Url {
        fn from(value: url::Url) -> Self {
            Self { value: value.to_string() }
        }
    }

    impl TryFrom<Url> for url::Url {
        type Error = ConversionError;

        fn try_from(value: Url) -> Result<Self, Self::Error> {
            type ErrorBuilder = ConversionErrorBuilder<Url, url::Url>;

            url::Url::parse(&value.value)
                .map_err(|cause| ErrorBuilder::new(format!("Url could not be parsed: {}", cause)))
        }
    }
}

pub mod vpn {
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
}
