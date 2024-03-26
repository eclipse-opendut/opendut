use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures::future::join_all;
use futures::FutureExt;
use tracing::{debug, error};

use opendut_carl_api::carl::cluster::{DeleteClusterDeploymentError, StoreClusterDeploymentError};
use opendut_types::cluster::{ClusterAssignment, ClusterConfiguration, ClusterDeployment, ClusterId, ClusterName, PeerClusterAssignment};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::peer::state::PeerState;
use opendut_types::topology::DeviceId;
use opendut_types::util::net::NetworkInterfaceDescriptor;
use opendut_types::util::Port;

use crate::actions;
use crate::actions::{AssignClusterParams, ListPeerDescriptorsParams};
use crate::peer::broker::PeerMessagingBrokerRef;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;

pub type ClusterManagerRef = Arc<ClusterManager>;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum DeployClusterError {
    #[error("Cluster <{0}> not found!")]
    ClusterConfigurationNotFound(ClusterId),
    #[error("A peer for device <{device_id}> of cluster '{cluster_name}' <{cluster_id}> not found.")]
    PeerForDeviceNotFound {
        device_id: DeviceId,
        cluster_id: ClusterId,
        cluster_name: ClusterName,
    },
    #[error("Peer designated as leader <{leader_id}> of cluster '{cluster_name}' <{cluster_id}> not found.")]
    LeaderNotFound {
        leader_id: PeerId,
        cluster_id: ClusterId,
        cluster_name: ClusterName,
    },
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
    options: ClusterManagerOptions,
}

impl ClusterManager {
    pub fn new(
        resources_manager: ResourcesManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        vpn: Vpn,
        options: ClusterManagerOptions,
    ) -> ClusterManagerRef {
        Arc::new(Self {
            resources_manager,
            peer_messaging_broker,
            vpn,
            options,
        })
    }
    #[tracing::instrument(skip(self), level="trace")]
    pub async fn deploy(&self, cluster_id: ClusterId) -> Result<(), DeployClusterError> {

        let cluster_config = self.resources_manager.resources(|resources| {
            resources.get::<ClusterConfiguration>(cluster_id)
        }).await
        .ok_or(DeployClusterError::ClusterConfigurationNotFound(cluster_id))?;

        let cluster_name = cluster_config.name;

        let all_peers = actions::list_peer_descriptors(ListPeerDescriptorsParams {
            resources_manager: Arc::clone(&self.resources_manager),
        }).await.map_err(|cause| DeployClusterError::Internal { cluster_id, cause: cause.to_string() })?;


        let member_interface_mapping = determine_member_interface_mapping(cluster_config.devices, all_peers, cluster_config.leader)
            .map_err(|cause| match cause {
                DetermineMemberInterfaceMappingError::PeerForDeviceNotFound { device_id } => DeployClusterError::PeerForDeviceNotFound { device_id, cluster_id, cluster_name },
            })?;

        let member_ids = member_interface_mapping.keys().cloned().collect::<Vec<_>>();

        if let Vpn::Enabled { vpn_client } = &self.vpn {
            vpn_client.create_cluster(cluster_id, &member_ids).await
                .map_err(|cause| {
                    let message = format!("Failure while creating cluster <{cluster_id}> in VPN service.");
                    error!("{}\n  {cause}", message);
                    DeployClusterError::Internal { cluster_id, cause: message }
                })?;

            let peers_string = member_ids.iter().map(|peer| peer.to_string()).collect::<Vec<_>>().join(",");
            debug!("Created group for cluster <{cluster_id}> in VPN service, using peers: {peers_string}");
        } else {
            debug!("VPN disabled. Not creating VPN group.")
        }

        let port_start = self.options.can_server_port_range_start;
        let port_end = self.options.can_server_port_range_start + u16::try_from(member_interface_mapping.len())
            .map_err(|cause| DeployClusterError::Internal { cluster_id, cause: cause.to_string() })?;
        let can_server_ports = (port_start..port_end)
            .map(Port)
            .collect::<Vec<_>>();

        let member_assignments: Vec<Result<PeerClusterAssignment, DeployClusterError>> = {
            let assignment_futures = std::iter::zip(member_interface_mapping, can_server_ports)
                .map(|((peer_id, device_interfaces), can_server_port)| {
                    self.resources_manager.get::<PeerState>(peer_id).map(move |peer_state: Option<PeerState>| {
                        let vpn_address = match peer_state {
                            Some(PeerState::Up { remote_host, .. }) => {
                                Ok(remote_host)
                            }
                            Some(_) => {
                                Err(DeployClusterError::Internal { cluster_id, cause: format!("Peer <{peer_id}> which is used in a cluster, should have a PeerState of 'Up'.") })
                            }
                            None => {
                                Err(DeployClusterError::Internal { cluster_id, cause: format!("Peer <{peer_id}> which is used in a cluster, should have a PeerState associated.") })
                            }
                        };
                        vpn_address.map(|vpn_address|
                            PeerClusterAssignment { peer_id, vpn_address, can_server_port, device_interfaces }
                        )
                    })
                })
                .collect::<Vec<_>>();

            join_all(assignment_futures).await
        };
        let member_assignments: Vec<PeerClusterAssignment> = member_assignments.into_iter().collect::<Result<_, _>>()?;

        for member_id in member_ids {
            actions::assign_cluster(AssignClusterParams {
                resources_manager: Arc::clone(&self.resources_manager),
                peer_messaging_broker: Arc::clone(&self.peer_messaging_broker),
                peer_id: member_id,
                cluster_assignment: ClusterAssignment {
                    id: cluster_id,
                    leader: cluster_config.leader,
                    assignments: member_assignments.clone(),
                },
            }).await
            .map_err(|cause| {
                let message = format!("Failure while assigning cluster <{cluster_id}> to peer <{member_id}>.");
                error!("{}\n  {cause}", message);
                DeployClusterError::Internal { cluster_id, cause: message }
            })?;
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn find_configuration(&self, id: ClusterId) -> Option<ClusterConfiguration> {
        self.resources_manager.resources(|resources| {
            resources.get::<ClusterConfiguration>(id)
        }).await
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn list_configuration(&self) -> Vec<ClusterConfiguration> {
        self.resources_manager.resources(|resources| {
            resources.iter::<ClusterConfiguration>().cloned().collect::<Vec<_>>()
        }).await
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn store_cluster_deployment(&self, deployment: ClusterDeployment) -> Result<ClusterId, StoreClusterDeploymentError> {
        let cluster_id = deployment.id;
        self.resources_manager.resources_mut(|resources| {
            resources.insert(deployment.id, deployment);
        }).await;
        if let Err(error) = self.deploy(cluster_id).await {
            error!("Failed to deploy cluster <{cluster_id}>, due to:\n  {error}");
        }
        Ok(cluster_id)
    }

    #[tracing::instrument(skip(self), level="trace")]
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

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn list_deployment(&self) -> Vec<ClusterDeployment> {
        self.resources_manager.resources(|resources| {
            resources.iter::<ClusterDeployment>().cloned().collect::<Vec<_>>()
        }).await
    }
}

fn determine_member_interface_mapping(
    cluster_devices: HashSet<DeviceId>,
    all_peers: Vec<PeerDescriptor>,
    leader: PeerId,
) -> Result<HashMap<PeerId, Vec<NetworkInterfaceDescriptor>>, DetermineMemberInterfaceMappingError> {

    let mut result: HashMap<PeerId, Vec<NetworkInterfaceDescriptor>> = HashMap::new();

    result.insert(leader, Vec::new()); //will later be replaced, if leader has devices

    for device_id in cluster_devices {
        let member_interfaces = all_peers.iter().find_map(|peer| {
            let interfaces: Vec<NetworkInterfaceDescriptor> = peer.topology.devices.iter()
                .filter(|device| device.id == device_id)
                .map(|device| device.interface.clone())
                .collect();

            if interfaces.is_empty() {
                None
            } else {
                Some((peer.id, interfaces))
            }
        });

        if let Some((peer, interfaces)) = member_interfaces {
            result.entry(peer)
                .or_default()
                .extend(interfaces);
        } else {
            return Err(DetermineMemberInterfaceMappingError::PeerForDeviceNotFound { device_id });
        }
    }
    Ok(result)
}

#[derive(Clone)]
pub struct ClusterManagerOptions {
    pub can_server_port_range_start: u16,
}
impl ClusterManagerOptions {
    pub fn load(config: &config::Config) -> Result<Self, opendut_util::settings::LoadError> {
        let can_server_port_range_start = config.get::<u16>("peer.can.server_port_range_start")?;

        Ok(ClusterManagerOptions {
            can_server_port_range_start,
        })
    }
}
#[derive(Debug, thiserror::Error)]
enum DetermineMemberInterfaceMappingError {
    #[error("Peer for device <{device_id}> not found.")]
    PeerForDeviceNotFound { device_id: DeviceId },
}


#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::time::Duration;

    use googletest::prelude::*;
    use rstest::{fixture, rstest};
    use tokio::sync::mpsc;

    use opendut_carl_api::proto::services::peer_messaging_broker::Downstream;
    use opendut_carl_api::proto::services::peer_messaging_broker::downstream;
    use opendut_types::cluster::ClusterName;
    use opendut_types::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkConfiguration};
    use opendut_types::peer::executor::{ContainerCommand, ContainerImage, ContainerName, Engine, ExecutorDescriptor, ExecutorDescriptors};
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceName};

    use crate::actions::{CreateClusterConfigurationParams, StorePeerDescriptorParams};
    use crate::peer::broker::{PeerMessagingBroker, PeerMessagingBrokerOptions};
    use crate::resources::manager::ResourcesManager;
    use crate::settings;

    use super::*;

    mod deploy_cluster {
        use opendut_carl_api::proto::services::peer_messaging_broker::ApplyPeerConfiguration;
        use opendut_types::peer::configuration::PeerConfiguration;

        use super::*;

        #[rstest]
        #[tokio::test]
        async fn deploy_cluster(
            fixture: Fixture,
            peer_a: PeerFixture,
            peer_b: PeerFixture,
        ) -> anyhow::Result<()> {

            let leader_id = peer_a.id;
            let cluster_id = ClusterId::random();
            let cluster_configuration = ClusterConfiguration {
                id: cluster_id,
                name: ClusterName::try_from("MyAwesomeCluster").unwrap(),
                leader: leader_id,
                devices: HashSet::from([peer_a.device, peer_b.device]),
            };

            actions::store_peer_descriptor(StorePeerDescriptorParams {
                resources_manager: Arc::clone(&fixture.resources_manager),
                vpn: Vpn::Disabled,
                peer_descriptor: Clone::clone(&peer_a.descriptor),
            }).await?;

            actions::store_peer_descriptor(StorePeerDescriptorParams {
                resources_manager: Arc::clone(&fixture.resources_manager),
                vpn: Vpn::Disabled,
                peer_descriptor: Clone::clone(&peer_b.descriptor),
            }).await?;


            let mut peer_a_rx = peer_open(peer_a.id, peer_a.remote_host, Arc::clone(&fixture.peer_messaging_broker)).await;
            let mut peer_b_rx = peer_open(peer_b.id, peer_b.remote_host, Arc::clone(&fixture.peer_messaging_broker)).await;


            actions::create_cluster_configuration(CreateClusterConfigurationParams {
                resources_manager: Arc::clone(&fixture.resources_manager),
                cluster_configuration,
            }).await?;

            assert_that!(fixture.testee.deploy(cluster_id).await, ok(eq(())));


            let expectation = || {
                matches_pattern!(ClusterAssignment {
                    id: eq(cluster_id),
                    leader: eq(leader_id),
                    assignments: any![
                        unordered_elements_are![
                            eq(PeerClusterAssignment {
                                peer_id: peer_a.id,
                                vpn_address: peer_a.remote_host,
                                can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start + 1),
                                device_interfaces: peer_a.descriptor.topology.devices.clone().into_iter().map(|device| device.interface).collect(),
                            }),
                            eq(PeerClusterAssignment {
                                peer_id: peer_b.id,
                                vpn_address: peer_b.remote_host,
                                can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start),
                                device_interfaces: peer_b.descriptor.topology.devices.clone().into_iter().map(|device| device.interface).collect(),
                            }),
                        ],
                        unordered_elements_are![
                            eq(PeerClusterAssignment {
                                peer_id: peer_a.id,
                                vpn_address: peer_a.remote_host,
                                can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start),
                                device_interfaces: peer_a.descriptor.topology.devices.into_iter().map(|device| device.interface).collect(),
                            }),
                            eq(PeerClusterAssignment {
                                peer_id: peer_b.id,
                                vpn_address: peer_b.remote_host,
                                can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start + 1),
                                device_interfaces: peer_b.descriptor.topology.devices.into_iter().map(|device| device.interface).collect(),
                            }),
                        ],
                    ]
                })
            };

            let result = receive_peer_configuration_message(&mut peer_a_rx).await;
            assert_that!(result.cluster_assignment.unwrap(), Clone::clone(&expectation)());

            let result = receive_peer_configuration_message(&mut peer_b_rx).await;
            assert_that!(result.cluster_assignment.unwrap(), expectation());

            Ok(())
        }

        async fn peer_open(peer_id: PeerId, peer_remote_host: IpAddr, peer_messaging_broker: PeerMessagingBrokerRef) -> mpsc::Receiver<Downstream> {
            let (_peer_tx, mut peer_rx) = peer_messaging_broker.open(peer_id, peer_remote_host).await;
            receive_peer_configuration_message(&mut peer_rx).await; //initial peer configuration after connect
            peer_rx
        }

        async fn receive_peer_configuration_message(peer_rx: &mut mpsc::Receiver<Downstream>) -> PeerConfiguration {
            let message = tokio::time::timeout(Duration::from_millis(500), peer_rx.recv()).await
                .unwrap().unwrap().message.unwrap();

            if let downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration { configuration: Some(peer_configuration) }) = message {
                peer_configuration.try_into().unwrap()
            } else {
                panic!("Did not receive valid message. Received this instead: {message:?}")
            }
        }
    }

    #[rstest]
    #[tokio::test]
    async fn deploy_should_fail_for_unknown_cluster(fixture: Fixture) -> anyhow::Result<()> {
        let unknown_cluster = ClusterId::random();

        assert_that!(
            fixture.testee.deploy(unknown_cluster).await,
            err(eq(DeployClusterError::ClusterConfigurationNotFound(unknown_cluster)))
        );

        Ok(())
    }

    #[rstest]
    fn should_determine_member_interface_mapping() -> anyhow::Result<()> {

        fn device(id: DeviceId, interface_name: NetworkInterfaceName) -> DeviceDescriptor {
            DeviceDescriptor {
                id,
                name: DeviceName::try_from(format!("test-device-{id}")).unwrap(),
                description: None,
                interface: NetworkInterfaceDescriptor {
                    name: interface_name,
                    configuration: NetworkInterfaceConfiguration::Ethernet,
                },
                tags: Vec::new(),
            }
        }

        let device_a = device(DeviceId::random(), NetworkInterfaceName::try_from("a")?);
        let device_b = device(DeviceId::random(), NetworkInterfaceName::try_from("b")?);
        let device_c = device(DeviceId::random(), NetworkInterfaceName::try_from("c")?);

        let cluster_devices = HashSet::from([device_a.id, device_b.id, device_c.id]);

        fn peer_descriptor(id: PeerId, devices: Vec<DeviceDescriptor>) -> PeerDescriptor {
            PeerDescriptor {
                id,
                name: PeerName::try_from(format!("peer-{id}")).unwrap(),
                location: PeerLocation::try_from("Ulm").ok(),
                network_configuration: PeerNetworkConfiguration {
                    interfaces: vec!(NetworkInterfaceDescriptor {
                        name: NetworkInterfaceName::try_from("eth0").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    })
                },
                topology: Topology {
                    devices,
                },
                executors: ExecutorDescriptors { executors: vec![] },
            }
        }

        let peer_1 = peer_descriptor(PeerId::random(), vec![device_a.clone()]);
        let peer_2 = peer_descriptor(PeerId::random(), vec![device_b.clone(), device_c.clone()]);
        let peer_leader = peer_descriptor(PeerId::random(), vec![]);


        let all_peers = vec![peer_1.clone(), peer_2.clone(), peer_leader.clone()];
        let leader = peer_leader.id;

        let result = determine_member_interface_mapping(cluster_devices, all_peers, leader)?;

        assert_that!(
            result,
            unordered_elements_are![
                (eq(peer_1.id), unordered_elements_are![eq(device_a.interface)]),
                (eq(peer_2.id), unordered_elements_are![eq(device_b.interface), eq(device_c.interface)]),
                (eq(peer_leader.id), empty()),
            ]
        );
        Ok(())
    }

    struct Fixture {
        testee: ClusterManagerRef,
        resources_manager: ResourcesManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        cluster_manager_options: ClusterManagerOptions,
    }
    #[fixture]
    fn fixture() -> Fixture {
        let settings = settings::load_defaults().unwrap();

        let resources_manager = ResourcesManager::new();
        let peer_messaging_broker = PeerMessagingBroker::new(
            Arc::clone(&resources_manager),
            PeerMessagingBrokerOptions::load(&settings.config).unwrap(),
        );

        let cluster_manager_options = ClusterManagerOptions::load(&settings.config).unwrap();

        let testee = ClusterManager::new(
            Arc::clone(&resources_manager),
            Arc::clone(&peer_messaging_broker),
            Vpn::Disabled,
            cluster_manager_options.clone(),
        );
        Fixture {
            testee,
            resources_manager,
            peer_messaging_broker,
            cluster_manager_options,
        }
    }

    #[fixture]
    fn peer_a() -> PeerFixture {
        peer_fixture("PeerA")
    }
    #[fixture]
    fn peer_b() -> PeerFixture {
        peer_fixture("PeerB")
    }

    struct PeerFixture {
        id: PeerId,
        device: DeviceId,
        remote_host: IpAddr,
        descriptor: PeerDescriptor,
    }
    fn peer_fixture(peer_name: &str) -> PeerFixture {
        let device = DeviceId::random();

        let id = PeerId::random();
        let remote_host = IpAddr::from_str("1.1.1.1").unwrap();
        let descriptor = PeerDescriptor {
            id,
            name: PeerName::try_from(peer_name).unwrap(),
            location: PeerLocation::try_from("Ulm").ok(),
            network_configuration: PeerNetworkConfiguration {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        name: NetworkInterfaceName::try_from("eth0").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    }
                ]
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: device,
                        name: DeviceName::try_from(format!("{peer_name}_Device_1")).unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: NetworkInterfaceDescriptor {
                            name: NetworkInterfaceName::try_from("eth0").unwrap(),
                            configuration: NetworkInterfaceConfiguration::Ethernet,
                        },
                        tags: vec![],
                    }
                ]
            },
            executors: ExecutorDescriptors {
                executors: vec![ExecutorDescriptor::Container {
                    engine: Engine::Docker,
                    name: ContainerName::Empty,
                    image: ContainerImage::try_from("testUrl").unwrap(),
                    volumes: vec![],
                    devices: vec![],
                    envs: vec![],
                    ports: vec![],
                    command: ContainerCommand::Default,
                    args: vec![],
                }],
            },
        };
        PeerFixture {
            id,
            device,
            remote_host,
            descriptor
        }
    }
}
