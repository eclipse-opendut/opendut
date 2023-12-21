use std::sync::Arc;

use opendut_carl_api::carl::cluster::{DeleteClusterDeploymentError, StoreClusterDeploymentError};
use opendut_carl_api::proto::services::peer_messaging_broker::AssignCluster;
use opendut_carl_api::proto::services::peer_messaging_broker::downstream::Message;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::proto::cluster::ClusterAssignment;

use crate::actions;
use crate::actions::ListPeerDescriptorsParams;
use crate::peer::broker::broker::PeerMessagingBrokerRef;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;

pub type ClusterManagerRef = Arc<ClusterManager>;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DeployClusterError {
    #[error("Cluster with id <{0}> not found!")]
    ClusterNotFound(ClusterId),
    #[error("An error occurred while deploying cluster <{cluster_id}>:\n  {cause}")]
    Internal {
        cluster_id: ClusterId,
        cause: String
    }
}

pub struct ClusterManager {
    resources_manager: ResourcesManagerRef,
    peer_messaging_broker: PeerMessagingBrokerRef,
    vpn: Vpn,
}

impl ClusterManager {
    pub fn new(
        resources_manager: ResourcesManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        vpn: Vpn
    ) -> Self {
        Self {
            resources_manager,
            peer_messaging_broker,
            vpn,
        }
    }

    pub async fn deploy(&self, cluster_id: ClusterId) -> Result<(), DeployClusterError> {
        let config = self.resources_manager.resources(|resources| {
            resources.get::<ClusterConfiguration>(cluster_id)
        }).await
        .ok_or(DeployClusterError::ClusterNotFound(cluster_id))?;

        let peers = actions::list_peer_descriptors(ListPeerDescriptorsParams {
            resources_manager: Arc::clone(&self.resources_manager),
        }).await.map_err(|cause| DeployClusterError::Internal { cluster_id, cause: cause.to_string() })?;

        let peers = peers.iter()
            .filter(|peer| peer.topology.devices.iter().any(|device| config.devices.contains(&device.id)))
            .map(|peer| peer.id)
            .collect::<Vec<_>>();

        if let Vpn::Enabled { vpn_client } = &self.vpn {
            vpn_client.create_cluster(cluster_id, &peers).await
                .unwrap(); // TODO: escalate error

            let peers_string = peers.iter().map(|peer| peer.to_string()).collect::<Vec<_>>().join(",");
            log::debug!("Created group for cluster <{cluster_id}> in VPN service, using peers: {peers_string}");
        } else {
            log::debug!("VPN disabled. Not creating VPN group.")
        }

        for peer in peers {
            self.peer_messaging_broker.send_to_peer(peer, Message::AssignCluster(AssignCluster {
                assignment: Some(ClusterAssignment {
                    id: Some(cluster_id.into()),
                    leader: None,
                    assignments: vec![],
                }),
            })).await.expect("Send message should be possible");
        }

        Ok(())
    }

    pub async fn find_configuration(&self, id: ClusterId) -> Option<ClusterConfiguration> {
        self.resources_manager.resources(|resources| {
            resources.get::<ClusterConfiguration>(id)
        }).await
    }

    pub async fn list_configuration(&self) -> Vec<ClusterConfiguration> {
        self.resources_manager.resources(|resources| {
            resources.iter::<ClusterConfiguration>().cloned().collect::<Vec<_>>()
        }).await
    }

    pub async fn store_cluster_deployment(&self, deployment: ClusterDeployment) -> Result<ClusterId, StoreClusterDeploymentError> {
        let cluster_id = deployment.id;
        self.resources_manager.resources_mut(|resources| {
            resources.insert(deployment.id, deployment);
        }).await;
        if let Err(error) = self.deploy(cluster_id).await {
            log::error!("Failed to deploy cluster <{cluster_id}>, due to:\n  {error}");
        }
        Ok(cluster_id)
    }

    pub async fn delete_cluster_deployment(&self, cluster_id: ClusterId) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {

        let (deployment, configuration) = self.resources_manager
            .resources_mut(|resources| {
                resources.remove::<ClusterDeployment>(cluster_id)
                    .map(|deployment| (deployment, resources.get::<ClusterConfiguration>(cluster_id)))
            })
            .await
            .ok_or(DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id })?;

        if let Some(configuration) = configuration {
            if let Vpn::Enabled { vpn_client } = &self.vpn {
                vpn_client.delete_cluster(cluster_id).await
                    .map_err(|error| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: configuration.name, cause: error.to_string() })?;
            }
        }

        Ok(deployment)
    }

    pub async fn find_deployment(&self, id: ClusterId) -> Option<ClusterDeployment> {
        self.resources_manager.resources(|resources| {
            resources.get::<ClusterDeployment>(id)
        }).await
    }

    pub async fn list_deployment(&self) -> Vec<ClusterDeployment> {
        self.resources_manager.resources(|resources| {
            resources.iter::<ClusterDeployment>().cloned().collect::<Vec<_>>()
        }).await
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::time::Duration;

    use googletest::prelude::*;

    use opendut_types::cluster::ClusterName;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerName};
    use opendut_types::topology::{Device, DeviceId, InterfaceName, Topology};

    use crate::actions::{CreateClusterConfigurationParams, StorePeerDescriptorParams};
    use crate::peer::broker::broker::PeerMessagingBroker;
    use crate::resources::manager::ResourcesManager;

    use super::*;

    #[tokio::test]
    async fn deploy_cluster() -> anyhow::Result<()> {

        let resources_manager = Arc::new(ResourcesManager::new());
        let broker = Arc::new(PeerMessagingBroker::new());

        let testee = ClusterManager::new(
            Arc::clone(&resources_manager),
            Arc::clone(&broker),
            Vpn::Disabled,
        );

        let peer_a_device_1 = DeviceId::random();
        let peer_b_device_1 = DeviceId::random();

        let peer_a_id = PeerId::random();
        let peer_a = PeerDescriptor {
            id: peer_a_id,
            name: PeerName::try_from("PeerA").unwrap(),
            topology: Topology {
                devices: vec![
                    Device {
                        id: peer_a_device_1,
                        name: String::from("PeerA Device 1"),
                        description: String::from("Huii"),
                        location: String::from("Ulm"),
                        interface: InterfaceName::try_from("eth0").unwrap(),
                        tags: vec![],
                    }
                ]
            },
        };

        let peer_b_id = PeerId::random();
        let peer_b = PeerDescriptor {
            id: peer_b_id,
            name: PeerName::try_from("PeerB").unwrap(),
            topology: Topology {
                devices: vec![
                    Device {
                        id: peer_b_device_1,
                        name: String::from("PeerB Device 1"),
                        description: String::from("Pfuii"),
                        location: String::from("New York"),
                        interface: InterfaceName::try_from("can1").unwrap(),
                        tags: vec![],
                    }
                ]
            },
        };

        let cluster_id = ClusterId::random();
        let cluster_configuration = ClusterConfiguration {
            id: cluster_id,
            name: ClusterName::try_from("MyAwesomeCluster").unwrap(),
            leader: peer_a_id,
            devices: HashSet::from([peer_a_device_1, peer_b_device_1]),
        };

        actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: Vpn::Disabled,
            peer_descriptor: peer_a,
        }).await?;

        actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: Vpn::Disabled,
            peer_descriptor: peer_b,
        }).await?;

        let (_peer_a_tx, mut peer_a_rx) = broker.open(peer_a_id).await;
        let (_peer_b_tx, mut peer_b_rx) = broker.open(peer_b_id).await;


        actions::create_cluster_configuration(CreateClusterConfigurationParams {
            resources_manager: Arc::clone(&resources_manager),
            cluster_configuration,
        }).await?;

        assert_that!(testee.deploy(cluster_id).await, ok(eq(())));

        let expected_assign_message = Message::AssignCluster(AssignCluster {
            assignment: Some(ClusterAssignment {
                id: Some(cluster_id.into()),
                leader: None,
                assignments: vec![],
            })
        });

        assert_that!(tokio::time::timeout(Duration::from_millis(500), peer_a_rx.recv()).await, ok(some(eq(Clone::clone(&expected_assign_message)))));
        assert_that!(tokio::time::timeout(Duration::from_millis(500), peer_b_rx.recv()).await, ok(some(eq(expected_assign_message))));

        Ok(())
    }

    #[tokio::test]
    async fn deploy_should_fail_for_unknown_cluster() -> anyhow::Result<()> {

        let resources_manager = Arc::new(ResourcesManager::new());
        let broker = Arc::new(PeerMessagingBroker::new());

        let testee = ClusterManager::new(
            Arc::clone(&resources_manager),
            Arc::clone(&broker),
            Vpn::Disabled,
        );

        let unknown_cluster = ClusterId::random();

        assert_that!(testee.deploy(unknown_cluster).await, err(eq(DeployClusterError::ClusterNotFound(unknown_cluster))));

        Ok(())
    }
}
