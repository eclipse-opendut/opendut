use crate::proto::topology::DeviceId;
use crate::proto::{conversion, ConversionError, ConversionResult};

crate::include_proto!("opendut.types.cluster");


conversion! {
    type Model = crate::cluster::ClusterId;
    type Proto = ClusterId;

    fn from(value: Model) -> Proto {
        Proto {
            uuid: Some(value.0.into())
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        extract!(value.uuid)
            .map(|uuid| crate::cluster::ClusterId(uuid.into()))
    }
}

conversion! {
    type Model = crate::cluster::ClusterName;
    type Proto = ClusterName;

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
    type Model = crate::cluster::ClusterDescriptor;
    type Proto = ClusterDescriptor;

    fn from(configuration: Model) -> Proto {
        Proto {
            id: Some(configuration.id.into()),
            name: Some(configuration.name.into()),
            leader: Some(configuration.leader.into()),
            devices: configuration.devices.into_iter()
                .map(DeviceId::from)
                .collect(),
        }
    }

    fn try_from(configuration: Proto) -> ConversionResult<Model> {
        let cluster_id: crate::cluster::ClusterId = extract!(configuration.id)?.try_into()?;

        let cluster_name: crate::cluster::ClusterName = extract!(configuration.name)?.try_into()?;

        let leader: crate::peer::PeerId = extract!(configuration.leader)?.try_into()?;

        Ok(Model {
            id: cluster_id,
            name: cluster_name,
            leader,
            devices: configuration.devices.into_iter()
                .map(DeviceId::try_into)
                .collect::<Result<_, _>>()?,
        })
    }
}

conversion! {
    type Model = crate::cluster::ClusterDeployment;
    type Proto = ClusterDeployment;

    fn from(deployment: Model) -> Proto {
        Proto {
            id: Some(deployment.id.into()),
        }
    }

    fn try_from(deployment: Proto) -> ConversionResult<Model> {
        let cluster_id: crate::cluster::ClusterId = extract!(deployment.id)?.try_into()?;

        Ok(Model {
            id: cluster_id,
        })
    }
}

conversion! {
    type Model = crate::cluster::state::ClusterState;
    type Proto = ClusterState;

    fn from(state: Model) -> Proto {
        match state {
            Model::Undeployed => {
                ClusterState {
                    inner: Some(cluster_state::Inner::Undeployed(ClusterStateUndeployed {}))
                }
            },
            Model::Deploying => {
                ClusterState {
                    inner: Some(cluster_state::Inner::Deploying(ClusterStateDeploying {}))
                }
            },
            Model::Deployed(inner) => {
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

    fn try_from(state: Proto) -> ConversionResult<Model> {
        let inner = extract!(state.inner)?;

        match inner {
            cluster_state::Inner::Undeployed(_) => {
                Ok(Model::Undeployed)
            }
            cluster_state::Inner::Deploying(_) => {
                Ok(Model::Deploying)
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
                Ok(Model::Deployed(inner))
            }
        }
    }
}

conversion! {
    type Model = crate::cluster::ClusterAssignment;
    type Proto = ClusterAssignment;

    fn from(value: Model) -> Proto {
        Proto {
            id: Some(value.id.into()),
            leader: Some(value.leader.into()),
            assignments: value.assignments.into_iter().map(Into::into).collect(),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let cluster_id: crate::cluster::ClusterId = extract!(value.id)?.try_into()?;

        let leader: crate::peer::PeerId = extract!(value.leader)?.try_into()?;

        let assignments: Vec<crate::cluster::PeerClusterAssignment> = value.assignments
            .into_iter()
            .map(TryInto::try_into)
            .collect::<Result<_, _>>()?;

        Ok(Model {
            id: cluster_id,
            leader,
            assignments,
        })
    }
}

conversion! {
    type Model = crate::cluster::PeerClusterAssignment;
    type Proto = PeerClusterAssignment;

    fn from(value: Model) -> Proto {
        Proto {
            peer_id: Some(value.peer_id.into()),
            vpn_address: Some(value.vpn_address.into()),
            can_server_port: Some(value.can_server_port.into()),
        }
    }

    fn try_from(value: Proto) -> ConversionResult<Model> {
        let peer_id: crate::peer::PeerId = extract!(value.peer_id)?.try_into()?;

        let vpn_address: std::net::IpAddr = extract!(value.vpn_address)?.try_into()?;

        let can_server_port: crate::util::Port = extract!(value.can_server_port)?.try_into()?;

        Ok(Model {
            peer_id,
            vpn_address,
            can_server_port,
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
                ok(eq(&native))
            );
        }

        { // Deploying
            let native = crate::cluster::state::ClusterState::Deploying;
            let proto: ClusterState = Clone::clone(&native).into();

            assert_that!(
                crate::cluster::state::ClusterState::try_from(Clone::clone(&proto)),
                ok(eq(&native))
            );
        }

        { // Deployed/Unhealthy
            let native = crate::cluster::state::ClusterState::Deployed(
                crate::cluster::state::DeployedClusterState::Unhealthy
            );
            let proto: ClusterState = Clone::clone(&native).into();

            assert_that!(
                crate::cluster::state::ClusterState::try_from(Clone::clone(&proto)),
                ok(eq(&native))
            );
        }

        { // Deployed/Healthy
            let native = crate::cluster::state::ClusterState::Deployed(
                crate::cluster::state::DeployedClusterState::Healthy
            );
            let proto: ClusterState = Clone::clone(&native).into();

            assert_that!(
                crate::cluster::state::ClusterState::try_from(Clone::clone(&proto)),
                ok(eq(&native))
            );
        }

        Ok(())
    }
}
