use crate::peer::broker::PeerMessagingBrokerRef;
use crate::persistence::error::PersistenceError;
use crate::resources::manager::ResourcesManagerRef;
use opendut_carl_api::proto::services::peer_messaging_broker::{downstream, ApplyPeerConfiguration};
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2};
use opendut_types::peer::state::{PeerBlockedState, PeerState, PeerUpState};
use opendut_types::peer::PeerId;

pub struct AssignClusterParams {
    pub resources_manager: ResourcesManagerRef,
    pub peer_messaging_broker: PeerMessagingBrokerRef,
    pub peer_id: PeerId,
    pub cluster_assignment: ClusterAssignment,
}

#[derive(thiserror::Error, Debug)]
pub enum AssignClusterError {
    #[error("Assigning cluster for peer <{0}> failed, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("Sending PeerConfiguration with ClusterAssignment to peer <{peer_id}> failed: {cause}")]
    SendingToPeerFailed { peer_id: PeerId, cause: String },
    #[error("Error while persisting ClusterAssignment for peer <{peer_id}>.")]
    Persistence { peer_id: PeerId, #[source] source: PersistenceError },
}

pub async fn assign_cluster(params: AssignClusterParams) -> Result<(), AssignClusterError> {

    let peer_id = params.peer_id;

    let (peer_configuration, peer_configuration2) = params.resources_manager.resources_mut(|resources| {
        let peer_configuration = resources.get::<PeerConfiguration>(peer_id)
            .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
            .ok_or(AssignClusterError::PeerNotFound(peer_id))
            .and_then(|peer_configuration| {
                let peer_configuration = PeerConfiguration {
                    cluster_assignment: Some(params.cluster_assignment),
                    ..peer_configuration
                };
                resources.insert(peer_id, Clone::clone(&peer_configuration))
                    .map_err(|source| AssignClusterError::Persistence { peer_id, source })?;
                Ok(peer_configuration)
            })?;

        let peer_configuration2 = resources.get::<PeerConfiguration2>(peer_id)
            .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
            .ok_or(AssignClusterError::PeerNotFound(peer_id))?;

        let peer_state = resources.get::<PeerState>(peer_id)
            .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
            .ok_or(AssignClusterError::PeerNotFound(peer_id))?;

        match peer_state {
            PeerState::Down => {}
            PeerState::Up { remote_host, .. } => {
                resources.insert(peer_id, PeerState::Up {
                    inner: PeerUpState::Blocked(PeerBlockedState::Member),
                    remote_host,
                })
                    .map_err(|source| AssignClusterError::Persistence { peer_id, source })?;
            }
        }

        Ok((peer_configuration, peer_configuration2))
    }).await?;

    params.peer_messaging_broker.send_to_peer(
        peer_id,
        downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
            configuration: Some(peer_configuration.into()),
            configuration2: Some(peer_configuration2.into()),
        }),
    ).await
    .map_err(|cause| AssignClusterError::SendingToPeerFailed {
        peer_id,
        cause: cause.to_string()
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::actions::peers::testing::{fixture, Fixture};
    use crate::peer::broker::{PeerMessagingBroker, PeerMessagingBrokerOptions};
    use googletest::prelude::*;
    use opendut_types::cluster::{ClusterAssignment, ClusterId};
    use opendut_types::peer::configuration::PeerNetworkConfiguration;
    use opendut_types::util::net::NetworkInterfaceName;
    use rstest::rstest;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::Arc;

    use super::*;

    #[rstest]
    #[tokio::test]
    async fn should_update_peer_configuration(fixture: Fixture) -> anyhow::Result<()> {

        let settings = crate::settings::load_defaults()?;
        let peer_id = fixture.peer_a_id;

        let resources_manager = fixture.resources_manager;
        let peer_messaging_broker = PeerMessagingBroker::new(
            Arc::clone(&resources_manager),
            PeerMessagingBrokerOptions::load(&settings.config).unwrap(),
        );

        let peer_configuration = PeerConfiguration {
            cluster_assignment: None,
            network: PeerNetworkConfiguration {
                bridge_name: NetworkInterfaceName::try_from("br-opendut-1").unwrap()
            }
        };
        resources_manager.resources_mut(|resources| {
            resources.insert(peer_id, Clone::clone(&peer_configuration))
        }).await?;
        let peer_configuration2 = PeerConfiguration2 {
            executors: vec![],
        };
        resources_manager.resources_mut(|resources| {
            resources.insert(peer_id, Clone::clone(&peer_configuration2))
        }).await?;

        let (_, mut receiver) = peer_messaging_broker.open(peer_id, IpAddr::from_str("1.2.3.4")?).await?;
        let received = receiver.recv().await.unwrap()
            .message.unwrap();
        assert_that!(
            received,
            eq(downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                configuration: Some(Clone::clone(&peer_configuration).into()),
                configuration2: Some(Clone::clone(&peer_configuration2).into()),
            }))
        );


        let cluster_assignment = ClusterAssignment {
            id: ClusterId::random(),
            leader: PeerId::random(),
            assignments: vec![],
        };


        assign_cluster(AssignClusterParams {
            resources_manager: Arc::clone(&resources_manager),
            peer_messaging_broker: Arc::clone(&peer_messaging_broker),
            peer_id,
            cluster_assignment: Clone::clone(&cluster_assignment),
        }).await?;


        let peer_configuration = PeerConfiguration {
            cluster_assignment: Some(cluster_assignment),
            ..peer_configuration
        };

        assert_that!(
            resources_manager.get::<PeerConfiguration>(peer_id).await?.as_ref(),
            some(eq(&peer_configuration))
        );


        let received = receiver.recv().await.unwrap()
            .message.unwrap();

        assert_that!(
            received,
            eq(downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                configuration: Some(Clone::clone(&peer_configuration).into()),
                configuration2: Some(peer_configuration2.into()),
            }))
        );

        Ok(())
    }
}
