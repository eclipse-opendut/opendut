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
            .ok_or(ErrorBuilder::field_not_set("uuid"))
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
            .map_err(|cause| ErrorBuilder::message(cause.to_string()))
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
            .ok_or(ErrorBuilder::field_not_set("id"))?
            .try_into()?;

        let cluster_name: crate::cluster::ClusterName = configuration.name
            .ok_or(ErrorBuilder::field_not_set("name"))?
            .try_into()?;

        let leader: crate::peer::PeerId = configuration.leader
            .ok_or(ErrorBuilder::field_not_set("leader"))?
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
            .ok_or(ErrorBuilder::field_not_set("id"))?
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
            .ok_or(ErrorBuilder::field_not_set("inner"))?;

        match inner {
            cluster_state::Inner::Undeployed(_) => {
                Ok(crate::cluster::state::ClusterState::Undeployed)
            }
            cluster_state::Inner::Deploying(_) => {
                Ok(crate::cluster::state::ClusterState::Deploying)
            }
            cluster_state::Inner::Deployed(state) => {
                let inner = state.inner
                    .ok_or(ErrorBuilder::field_not_set("inner"))?;
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

impl From<crate::cluster::ClusterAssignment> for ClusterAssignment {
    fn from(value: crate::cluster::ClusterAssignment) -> Self {
        Self {
            id: Some(value.id.into()),
            leader: Some(value.leader.into()),
            assignments: value.assignments.into_iter().map(Into::into).collect(),
        }
    }
}
impl TryFrom<ClusterAssignment> for crate::cluster::ClusterAssignment {
    type Error = ConversionError;

    fn try_from(value: ClusterAssignment) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<ClusterAssignment, crate::cluster::ClusterAssignment>;

        let cluster_id: crate::cluster::ClusterId = value.id
            .ok_or(ErrorBuilder::field_not_set("id"))?
            .try_into()?;

        let leader: crate::peer::PeerId = value.leader
            .ok_or(ErrorBuilder::field_not_set("leader"))?
            .try_into()?;

        let assignments: Vec<crate::cluster::PeerClusterAssignment> = value.assignments
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            id: cluster_id,
            leader,
            assignments,
        })
    }
}

impl From<crate::cluster::PeerClusterAssignment> for PeerClusterAssignment {
    fn from(value: crate::cluster::PeerClusterAssignment) -> Self {
        Self {
            peer_id: Some(value.peer_id.into()),
            vpn_address: Some(value.vpn_address.into()),
            can_server_port: Some(value.can_server_port.into()),
            device_interfaces: value.device_interfaces.into_iter().map(Into::into).collect(),
        }
    }
}
impl TryFrom<PeerClusterAssignment> for crate::cluster::PeerClusterAssignment {
    type Error = ConversionError;

    fn try_from(value: PeerClusterAssignment) -> Result<Self, Self::Error> {
        type ErrorBuilder = ConversionErrorBuilder<PeerClusterAssignment, crate::cluster::PeerClusterAssignment>;

        let peer_id: crate::peer::PeerId = value.peer_id
            .ok_or(ErrorBuilder::field_not_set("peer_id"))?
            .try_into()?;

        let vpn_address: std::net::IpAddr = value.vpn_address
            .ok_or(ErrorBuilder::field_not_set("vpn_address"))?
            .try_into()?;

        let can_server_port: crate::util::Port = value.can_server_port
            .ok_or(ErrorBuilder::field_not_set("can_server_port"))?
            .try_into()?;

        let device_interfaces: Vec<crate::util::net::NetworkInterfaceDescriptor> = value.device_interfaces
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        Ok(Self {
            peer_id,
            vpn_address,
            can_server_port,
            device_interfaces,
        })
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
