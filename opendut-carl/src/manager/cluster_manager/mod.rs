use anyhow::Context;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use futures::future::join_all;
use futures::FutureExt;
use tracing::{debug, error, trace, warn};

use opendut_model::cluster::{ClusterAssignment, ClusterDescriptor, ClusterDeployment, ClusterId, ClusterName, PeerClusterAssignment};
use opendut_model::peer::state::PeerConnectionState;
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::topology::{DeviceDescriptor, DeviceId};
use opendut_model::util::net::{NetworkInterfaceDescriptor, NetworkInterfaceName};
use opendut_model::util::Port;

use crate::manager::peer_messaging_broker::PeerMessagingBrokerRef;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::{MapErrToInner, PersistenceError, PersistenceResult};
use crate::resource::storage::ResourcesStorageApi;
use crate::settings::vpn::Vpn;

pub mod create_cluster_descriptor;
pub use create_cluster_descriptor::*;

pub mod delete_cluster_descriptor;
pub use delete_cluster_descriptor::*;

pub mod delete_cluster_deployment;
#[allow(unused)]
pub use delete_cluster_deployment::*;

pub mod list_cluster_peer_states;
pub use list_cluster_peer_states::*;

pub mod list_cluster_peers;
pub use list_cluster_peers::*;

pub mod list_deployed_clusters;
mod effects;

use crate::manager::peer_manager::{AssignClusterOptions, AssignClusterParams};
pub use list_deployed_clusters::*;

pub type ClusterManagerRef = Arc<Mutex<ClusterManager>>;

use error::*;


pub struct ClusterManager {
    resource_manager: ResourceManagerRef,
    peer_messaging_broker: PeerMessagingBrokerRef,
    pub vpn: Vpn,
    options: ClusterManagerOptions,
    can_server_port_counter: u16,
}

impl ClusterManager {
    pub async fn create(
        resource_manager: ResourceManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        vpn: Vpn,
        options: ClusterManagerOptions,
    ) -> ClusterManagerRef {
        let can_server_port_counter = options.can_server_port_range_start;

        let self_ref = Arc::new(Mutex::new(Self {
            resource_manager: Arc::clone(&resource_manager),
            peer_messaging_broker,
            vpn,
            options,
            can_server_port_counter
        }));

        effects::register(resource_manager.clone(), self_ref.clone()).await;

        self_ref
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn get_cluster_descriptor(&self, cluster_id: ClusterId) -> Result<Option<ClusterDescriptor>, GetClusterDescriptorError> {
        self.resource_manager.get::<ClusterDescriptor>(cluster_id).await
            .map_err(|source| GetClusterDescriptorError { cluster_id, source })
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn list_cluster_descriptor(&self) -> Result<Vec<ClusterDescriptor>, ListClusterDescriptorsError> {
        self.resource_manager.list::<ClusterDescriptor>().await
            .map(|clusters| clusters.into_values().collect::<Vec<_>>())
            .map_err(|source| ListClusterDescriptorsError { source })
    }


    #[tracing::instrument(skip(self), level="trace")]
    pub async fn store_cluster_deployment(&mut self, deployment: ClusterDeployment) -> Result<ClusterId, StoreClusterDeploymentError> {
        let cluster_id = deployment.id;

        let cluster_peers =
            self.resource_manager.resources(async |resources| {
                resources.list_cluster_peer_states(cluster_id).await
            }).await
            .map_err_to_inner(|source| ListClusterPeerStatesError::Persistence { cluster_id, source })
            .map_err(|source| StoreClusterDeploymentError::ListClusterPeerStates { cluster_id, source })?;

        let cluster_deployable = cluster_peers.check_all_peers_are_available_not_necessarily_online();
        match cluster_deployable {
            ClusterDeployable::AllPeersAvailable => {
                self.resource_manager.resources_mut(async |resources| {
                    let cluster_name = resources.get::<ClusterDescriptor>(cluster_id)
                        .map_err(|source| StoreClusterDeploymentError::Persistence { cluster_id, cluster_name: None, source })?
                        .map(|cluster| cluster.name)
                        .unwrap_or_else(|| ClusterName::try_from("unknown_cluster").unwrap());

                    resources.insert(cluster_id, deployment)
                        .map_err(|source| StoreClusterDeploymentError::Persistence { cluster_id, cluster_name: Some(cluster_name.clone()), source })
                }).await
                    .map_err(|source| StoreClusterDeploymentError::Persistence { cluster_id, cluster_name: None, source })??;
            }
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers } => {
                let blocked_peers_by_id = unavailable_peers.into_iter().collect::<Vec<_>>();
                warn!("Cannot store cluster deployment, because the following peers are blocked already: {blocked_peers_by_id:?}");
                return Err(StoreClusterDeploymentError::IllegalPeerState { cluster_id: cluster_peers.cluster_id, cluster_name: None, invalid_peers: blocked_peers_by_id });
            }
            ClusterDeployable::AlreadyDeployed => {
                trace!("Received instruction to store deployment for cluster <{cluster_id}>, which already exists. Ignoring.");
            }
        }

        if let Err(error) = self.rollout_cluster_if_all_peers_available(cluster_id).await {
            error!("Failed to deploy cluster <{cluster_id}> after storing cluster deployment, despite all peers being available, due to:\n  {error}");
        }
        Ok(cluster_id)
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn get_cluster_deployment(&self, cluster_id: ClusterId) -> Result<Option<ClusterDeployment>, GetClusterDeploymentError> {
        self.resource_manager.get::<ClusterDeployment>(cluster_id).await
            .map_err(|source| GetClusterDeploymentError::Persistence { cluster_id, source })
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn list_cluster_deployment(&self) -> Result<Vec<ClusterDeployment>, ListClusterDeploymentsError> {
        self.resource_manager.list::<ClusterDeployment>().await
            .map(|clusters| clusters.into_values().collect::<Vec<_>>())
            .map_err(|source| ListClusterDeploymentsError { source })
    }

    async fn rollout_all_clusters_containing_newly_available_peer(&mut self, peer_id: PeerId) -> anyhow::Result<()> {

        let clusters_containing_devices_of_upped_peer = self.resource_manager.resources_mut(async |resources| {

            let peer_descriptor = resources.get::<PeerDescriptor>(peer_id)?
                .context(format!("No peer descriptor found for newly available peer <{peer_id}>."))?;

            let peer_devices = peer_descriptor.topology.devices
                .into_iter()
                .map(|device| device.id)
                .collect::<Vec<_>>();

            let cluster_descriptors = resources.list::<ClusterDescriptor>()?;

            let clusters_containing_devices_of_upped_peer = cluster_descriptors.into_iter()
                .filter(|(_, cluster_descriptor)|
                    cluster_descriptor.devices.iter()
                        .any(|device| peer_devices.contains(device))
                )
                .filter_map(|(cluster_id, _)| { //filter out clusters without stored deployment
                    resources.get::<ClusterDeployment>(cluster_id)
                        .transpose()
                })
                .collect::<Result<Vec<_>, _>>()?;

            anyhow::Ok(clusters_containing_devices_of_upped_peer)
        }).await??;


        if clusters_containing_devices_of_upped_peer.is_empty() {
            trace!("Devices of newly available peer <{peer_id}> are not used in any clusters. Not deploying any clusters.");
        } else {
            for cluster in clusters_containing_devices_of_upped_peer {
                self.rollout_cluster_if_all_peers_available(cluster.id).await?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self), level="trace")]
    pub async fn rollout_cluster_if_all_peers_available(&mut self, cluster_id: ClusterId) -> Result<(), RolloutClusterError> {
        let cluster_peer_states = self.resource_manager.resources(async |resources| {
            resources.list_cluster_peer_states(cluster_id).await
        }).await
        .map_err_to_inner(|source| ListClusterPeerStatesError::Persistence { cluster_id, source })
        .map_err(|source| RolloutClusterError::ListClusterPeerStates { cluster_id, source })?;

        let cluster_deployable = cluster_peer_states.check_cluster_deployable();

        match cluster_deployable {
            ClusterDeployable::AllPeersAvailable => {
                debug!("All peers of cluster <{cluster_id}> are now available. Deploying...");
                self.rollout_cluster(cluster_id).await?;
            }
            ClusterDeployable::AlreadyDeployed => {
                debug!("Cluster <{cluster_id}> is already deployed. Triggering new deployment anyway.");  // TODO: re-evaluate if we can do this differently
                self.rollout_cluster(cluster_id).await?;
            }
            ClusterDeployable::NotAllPeersAvailable { unavailable_peers } => {
                debug!(
                    "Not all peers of cluster <{cluster_id}> are available, so not deploying. Unavailable peers: {}",
                    unavailable_peers.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        Ok(())
    }


    #[tracing::instrument(skip(self), level="debug")]
    async fn rollout_cluster(&mut self, cluster_id: ClusterId) -> Result<(), RolloutClusterError> {

        let cluster_config = self.resource_manager.get::<ClusterDescriptor>(cluster_id).await
            .map_err(|source| RolloutClusterError::Persistence { cluster_id, source })?
            .ok_or(RolloutClusterError::ClusterDescriptorNotFound(cluster_id))?;

        let cluster_name = cluster_config.name;

        let all_peers = self.resource_manager.list::<PeerDescriptor>().await
            .map_err(|source| RolloutClusterError::Persistence { cluster_id, source })?
            .into_values()
            .collect::<Vec<_>>();


        let member_interface_mapping = determine_member_interface_mapping(cluster_config.devices, all_peers, cluster_config.leader)
            .map_err(|cause| match cause {
                DetermineMemberInterfaceMappingError::PeerForDeviceNotFound { device_id } => RolloutClusterError::PeerForDeviceNotFound { device_id, cluster_id, cluster_name },
            })?;

        let member_ids = member_interface_mapping.keys().copied().collect::<Vec<_>>();

        if let Vpn::Enabled { vpn_client } = &self.vpn {
            vpn_client.create_cluster(cluster_id, &member_ids).await
                .map_err(|cause| {
                    let message = format!("Failure while creating cluster <{cluster_id}> in VPN service.");
                    error!("{}\n  {cause}", message);
                    RolloutClusterError::Internal { cluster_id, cause: message }
                })?;

            let peers_string = member_ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",");
            debug!("Created group for cluster <{cluster_id}> in VPN service, using peers: {peers_string}");
        } else {
            debug!("VPN disabled. Not creating VPN group.");
        }

        let can_server_ports = self.determine_can_server_ports(&member_ids, cluster_id)?;

        let member_assignments: Vec<Result<(PeerId, PeerClusterAssignment), RolloutClusterError>> = {
            let assignment_futures = std::iter::zip(member_ids, can_server_ports)
                .map(|(peer_id, can_server_port)| {
                    self.resource_manager.get::<PeerConnectionState>(peer_id)
                        .map(move |peer_connection_state: PersistenceResult<Option<PeerConnectionState>>| {
                            let vpn_address = match peer_connection_state {
                                Ok(peer_connection_state) => match peer_connection_state {
                                    Some(PeerConnectionState::Online { remote_host }) => {
                                        Ok(remote_host)
                                    }
                                    Some(PeerConnectionState::Offline) => {
                                        Err(RolloutClusterError::Internal { cluster_id, cause: format!("Peer <{peer_id}> which is used in a cluster, should have a PeerConnectionState of 'Online'.") })
                                    }
                                    None => {
                                        Err(RolloutClusterError::Internal { cluster_id, cause: format!("Peer <{peer_id}> which is used in a cluster, should have a PeerConnectionState associated.") })
                                    }
                                }
                                Err(cause) => {
                                    let message = format!("Error while accessing persistence to read PeerConnectionState of peer <{peer_id}>");
                                    error!("{message}:\n  {cause}");
                                    Err(RolloutClusterError::Internal { cluster_id, cause: message })
                                }
                            };

                            vpn_address.map(|vpn_address|
                                (peer_id, PeerClusterAssignment { vpn_address, can_server_port })
                            )
                        })
                })
                .collect::<Vec<_>>();

            join_all(assignment_futures).await
        };
        let member_assignments: HashMap<PeerId, PeerClusterAssignment> = member_assignments.into_iter().collect::<Result<_, _>>()?;


        let assign_cluster_options = AssignClusterOptions {
            bridge_name_default: self.options.bridge_name_default.clone(),
        };

        self.resource_manager.resources_mut(async |resources| {
            for (member_id, device_interfaces) in member_interface_mapping {

                resources.assign_cluster(AssignClusterParams {
                    peer_messaging_broker: Arc::clone(&self.peer_messaging_broker),
                    peer_id: member_id,
                    cluster_assignment: ClusterAssignment {
                        id: cluster_id,
                        leader: cluster_config.leader,
                        assignments: member_assignments.clone(),
                    },
                    device_interfaces,
                    options: assign_cluster_options.clone(),
                }).await
                .map_err(|cause| {
                    let message = format!("Failure while assigning cluster <{cluster_id}> to peer <{member_id}>. Cause: {cause}"); // TODO
                    error!("{}\n  {cause}", message);
                    RolloutClusterError::Internal { cluster_id, cause: message }
                })?;
            }
            Ok(())
        }).await
        .map_err(|source| RolloutClusterError::Persistence { cluster_id, source: source.context("Error when closing transaction while assigning peers to clusters") })??;

        Ok(())
    }

    fn determine_can_server_ports(&mut self, member_interface_mapping: &[PeerId], cluster_id: ClusterId) -> Result<Vec<Port>, RolloutClusterError> {
        let n_peers = u16::try_from(member_interface_mapping.len())
            .map_err(|cause| RolloutClusterError::DetermineCanServerPort { cluster_id, cause: cause.to_string() })?;

        if self.options.can_server_port_range_start + n_peers >= self.options.can_server_port_range_end {
            return Err(RolloutClusterError::DetermineCanServerPort {
                cluster_id,
                cause: format!(
                    "Failure while creating cluster <{}>. Port range [{}, {}) specified by 'can_server_port_range_start' \
                    and 'can_server_port_range_start' is too narrow for the configured number of peers ({})",
                    cluster_id,
                    self.options.can_server_port_range_start,
                    self.options.can_server_port_range_end,
                    n_peers,
                )
            })
        } else if self.options.can_server_port_range_start + n_peers * 2 >= self.options.can_server_port_range_end {
            warn!(
                "Port range [{}, {}) specified by 'can_server_port_range_start' \
                and 'can_server_port_range_start' is very narrow for the configured number of peers ({}). This may cause errors on EDGAR.",
                self.options.can_server_port_range_start,
                self.options.can_server_port_range_end,
                n_peers
            );
        }

        // Wrap-around the counter when we reached the end of the range of usable ports
        if self.can_server_port_counter + n_peers >= self.options.can_server_port_range_end {
            self.can_server_port_counter = self.options.can_server_port_range_start;
        }

        let can_server_ports = (self.can_server_port_counter..self.can_server_port_counter + n_peers)
            .map(Port)
            .collect::<Vec<_>>();

        self.can_server_port_counter += n_peers;

        Ok(can_server_ports)
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

            let devices: Vec<DeviceDescriptor> = peer.topology.devices.iter()
                .filter(|device| device.id == device_id)
                .cloned()
                .collect();

            if devices.is_empty() {
                None
            } else {
                let interfaces = peer.network.interfaces_zipped_with_devices(&devices)
                    .into_iter()
                    .map(|(interface, _)| interface)
                    .collect::<Vec<_>>();

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
    pub can_server_port_range_end: u16,
    pub bridge_name_default: NetworkInterfaceName,
}
impl ClusterManagerOptions {
    pub fn load(config: &config::Config) -> Result<Self, opendut_util::settings::LoadError> {
        let can_server_port_range_start = config.get::<u16>("peer.can.server_port_range_start")?;
        let can_server_port_range_end = config.get::<u16>("peer.can.server_port_range_end")?;

        let field = "peer.ethernet.bridge.name.default";
        let bridge_name_default = config.get_string(field)
            .map_err(|cause| opendut_util::settings::LoadError::ReadField { field, source: cause.into() })?;

        let bridge_name_default = NetworkInterfaceName::try_from(bridge_name_default.clone())
            .map_err(|cause| opendut_util::settings::LoadError::ParseValue { field, value: bridge_name_default, source: cause.into() })?;

        Ok(ClusterManagerOptions {
            can_server_port_range_start,
            can_server_port_range_end,
            bridge_name_default,
        })
    }
}
#[derive(Debug, thiserror::Error)]
enum DetermineMemberInterfaceMappingError {
    #[error("Peer for device <{device_id}> not found.")]
    PeerForDeviceNotFound { device_id: DeviceId },
}

pub mod error {
    use super::*;
    use opendut_model::cluster::ClusterDisplay;

    #[derive(thiserror::Error, Debug)]
    #[error("ClusterDescriptor <{cluster_id}> could not be retrieved")]
    pub struct GetClusterDescriptorError {
        pub cluster_id: ClusterId,
        #[source] pub source: PersistenceError,
    }

    #[derive(thiserror::Error, Debug)]
    #[error("Error while listing cluster descriptors")]
    pub struct ListClusterDescriptorsError {
        pub source: PersistenceError,
    }

    #[derive(thiserror::Error, Debug)]
    #[error("Error while storing cluster deployment for cluster {cluster_id}")]
    pub enum StoreClusterDeploymentError {
        #[error("ClusterDeployment for cluster {cluster} failed, due to down or already in use peers: {invalid_peers:?}", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
        IllegalPeerState {
            cluster_id: ClusterId,
            cluster_name: Option<ClusterName>,
            invalid_peers: Vec<PeerId>,
        },
        ListClusterPeerStates { cluster_id: ClusterId, #[source] source: ListClusterPeerStatesError },
        Persistence { cluster_id: ClusterId, cluster_name: Option<ClusterName>, #[source] source: PersistenceError },
    }

    #[derive(thiserror::Error, Debug)]
    pub enum GetClusterDeploymentError {
        #[error("Error when accessing persistence while retrieving cluster deployment for cluster <{cluster_id}>")]
        Persistence {
            cluster_id: ClusterId,
            #[source] source: PersistenceError,
        },
    }

    #[derive(thiserror::Error, Debug)]
    #[error("Error while listing cluster deployments")]
    pub struct ListClusterDeploymentsError {
        #[source] pub source: PersistenceError,
    }

    #[derive(thiserror::Error, Debug)]
    pub enum RolloutClusterError {
        #[error("Cluster <{0}> not found!")]
        ClusterDescriptorNotFound(ClusterId),
        #[error("A peer for device <{device_id}> of cluster '{cluster_name}' <{cluster_id}> not found.")]
        PeerForDeviceNotFound {
            device_id: DeviceId,
            cluster_id: ClusterId,
            cluster_name: ClusterName,
        },
        #[error("Error when listing cluster peer states while rolling out cluster <{cluster_id}>")]
        ListClusterPeerStates {
            cluster_id: ClusterId,
            #[source] source: ListClusterPeerStatesError,
        },
        #[error("Error when accessing persistence while rolling out cluster <{cluster_id}>")]
        Persistence {
            cluster_id: ClusterId,
            #[source] source: PersistenceError,
        },
        #[error("Error when determining CAN server port while rolling out cluster <{cluster_id}>")]
        DetermineCanServerPort {
            cluster_id: ClusterId,
            cause: String,
        },
        #[error("Internal error while rolling out cluster <{cluster_id}>:\n  {cause}")]
        Internal {
            cluster_id: ClusterId,
            cause: String,
        },
    }
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

    use opendut_model::cluster::ClusterName;
    use opendut_model::peer::executor::{container::{ContainerCommand, ContainerImage, ContainerName, Engine}, ExecutorDescriptor, ExecutorDescriptors, ExecutorId, ExecutorKind};
    use opendut_model::peer::{PeerDescriptor, PeerId, PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_model::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, Topology};
    use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceId, NetworkInterfaceName};

    use crate::manager::peer_messaging_broker::{PeerMessagingBroker, PeerMessagingBrokerOptions};
    use crate::resource::manager::ResourceManager;
    use crate::settings;

    use super::*;

    mod rollout_cluster {
        use super::*;
        use crate::manager::peer_manager::StorePeerDescriptorParams;
        use opendut_carl_api::carl::broker::{stream_header, DownstreamMessage, DownstreamMessagePayload};
        use opendut_model::peer::configuration::{OldPeerConfiguration, PeerConfiguration};

        #[rstest]
        #[tokio::test]
        async fn rollout_cluster(
            peer_a: PeerFixture,
            peer_b: PeerFixture,
        ) -> anyhow::Result<()> {
            let fixture = Fixture::create().await;

            let leader_id = peer_a.id;
            let cluster_id = ClusterId::random();
            let cluster_descriptor = ClusterDescriptor {
                id: cluster_id,
                name: ClusterName::try_from("MyAwesomeCluster").unwrap(),
                leader: leader_id,
                devices: HashSet::from([peer_a.device, peer_b.device]),
            };

            fixture.resource_manager.resources_mut::<_, (), anyhow::Error>(async |resources| {
                resources.store_peer_descriptor(StorePeerDescriptorParams {
                    vpn: Vpn::Disabled,
                    peer_descriptor: Clone::clone(&peer_a.descriptor),
                }).await?;

                resources.store_peer_descriptor(StorePeerDescriptorParams {
                    vpn: Vpn::Disabled,
                    peer_descriptor: Clone::clone(&peer_b.descriptor),
                }).await?;

                Ok(())
            }).await??;


            let mut peer_a_rx = peer_open(peer_a.id, peer_a.remote_host, Arc::clone(&fixture.peer_messaging_broker)).await?;
            let mut peer_b_rx = peer_open(peer_b.id, peer_b.remote_host, Arc::clone(&fixture.peer_messaging_broker)).await?;


            fixture.resource_manager.resources_mut(async |resources| {
                resources.create_cluster_descriptor(CreateClusterDescriptorParams {
                    cluster_descriptor,
                })
            }).await??;

            assert_that!(fixture.testee.lock().await.rollout_cluster(cluster_id).await, ok(eq(&())));


            let assert_cluster_assignment_valid = |cluster_assignment: &ClusterAssignment| {
                assert_that!(
                    cluster_assignment,
                    matches_pattern!(ClusterAssignment {
                        id: &cluster_id,
                        leader: &leader_id,
                        assignments: any![
                            unordered_elements_are![
                                (&peer_a.id, &PeerClusterAssignment {
                                    vpn_address: peer_a.remote_host,
                                    can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start + 1),
                                }),
                                (&peer_b.id, &PeerClusterAssignment {
                                    vpn_address: peer_b.remote_host,
                                    can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start),
                                }),
                            ],
                            unordered_elements_are![
                                (&peer_a.id, &PeerClusterAssignment {
                                    vpn_address: peer_a.remote_host,
                                    can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start),
                                }),
                                (&peer_b.id, &PeerClusterAssignment {
                                    vpn_address: peer_b.remote_host,
                                    can_server_port: Port(fixture.cluster_manager_options.can_server_port_range_start + 1),
                                }),
                            ],
                        ]
                    })
                );
            };


            let (result, _result2) = receive_peer_configuration_message(&mut peer_a_rx).await;
            assert_cluster_assignment_valid(&result.cluster_assignment.unwrap());

            let (result, _result2) = receive_peer_configuration_message(&mut peer_b_rx).await;
            assert_cluster_assignment_valid(&result.cluster_assignment.unwrap());

            Ok(())
        }

        async fn peer_open(peer_id: PeerId, peer_remote_host: IpAddr, peer_messaging_broker: PeerMessagingBrokerRef) -> anyhow::Result<mpsc::Receiver<DownstreamMessage>> {
            let (_peer_tx, mut peer_rx) = peer_messaging_broker.open(peer_id, peer_remote_host, stream_header::ExtraHeaders::default()).await?;
            receive_peer_configuration_message(&mut peer_rx).await; //initial peer configuration after connect
            Ok(peer_rx)
        }

        async fn receive_peer_configuration_message(peer_rx: &mut mpsc::Receiver<DownstreamMessage>) -> (OldPeerConfiguration, PeerConfiguration) {
            let message = tokio::time::timeout(Duration::from_millis(500), peer_rx.recv()).await
                .unwrap().unwrap().payload;

            if let DownstreamMessagePayload::ApplyPeerConfiguration(peer_config) = message {
                (peer_config.old_configuration, peer_config.configuration)
            } else {
                panic!("Did not receive valid message. Received this instead: {message:?}")
            }
        }
    }

    #[tokio::test]
    async fn rollout_should_fail_for_unknown_cluster() -> anyhow::Result<()> {
        let fixture = Fixture::create().await;
        let unknown_cluster = ClusterId::random();

        let result = fixture.testee.lock().await.rollout_cluster(unknown_cluster).await;

        let Err(RolloutClusterError::ClusterDescriptorNotFound(cluster)) = result
        else { panic!("Result is not a ClusterDescriptorNotFoundError.") };

        assert_eq!(cluster, unknown_cluster);

        Ok(())
    }

    #[test]
    fn should_determine_member_interface_mapping() -> anyhow::Result<()> {

        fn device_and_interface(id: DeviceId, interface_name: NetworkInterfaceName) -> (DeviceDescriptor, NetworkInterfaceDescriptor) {
            let network_interface = NetworkInterfaceDescriptor {
                id: NetworkInterfaceId::random(),
                name: interface_name,
                configuration: NetworkInterfaceConfiguration::Ethernet,
            };
            let device = DeviceDescriptor {
                id,
                name: DeviceName::try_from(format!("test-device-{id}")).unwrap(),
                description: None,
                interface: network_interface.id,
                tags: Vec::new(),
            };
            (device, network_interface)
        }

        let (device_a, interface_a) = device_and_interface(DeviceId::random(), NetworkInterfaceName::try_from("a")?);
        let (device_b, interface_b) = device_and_interface(DeviceId::random(), NetworkInterfaceName::try_from("b")?);
        let (device_c, interface_c) = device_and_interface(DeviceId::random(), NetworkInterfaceName::try_from("c")?);

        let cluster_devices = HashSet::from([device_a.id, device_b.id, device_c.id]);

        fn peer_descriptor(id: PeerId, devices: Vec<DeviceDescriptor>, interfaces: Vec<NetworkInterfaceDescriptor>) -> PeerDescriptor {
            PeerDescriptor {
                id,
                name: PeerName::try_from(format!("peer-{id}")).unwrap(),
                location: PeerLocation::try_from("Ulm").ok(),
                network: PeerNetworkDescriptor {
                    interfaces,
                    bridge_name: Some(NetworkInterfaceName::try_from("br-custom").unwrap()),
                },
                topology: Topology {
                    devices,
                },
                executors: ExecutorDescriptors { executors: vec![] },
            }
        }

        let peer_1 = peer_descriptor(PeerId::random(), vec![device_a.clone()], vec![interface_a.clone()]);
        let peer_2 = peer_descriptor(PeerId::random(), vec![device_b.clone(), device_c.clone()], vec![interface_b.clone(), interface_c.clone()]);
        let peer_leader = peer_descriptor(PeerId::random(), vec![], vec![]);


        let all_peers = vec![peer_1.clone(), peer_2.clone(), peer_leader.clone()];
        let leader = peer_leader.id;

        let result = determine_member_interface_mapping(cluster_devices, all_peers, leader)?;

        assert_that!(
            result,
            unordered_elements_are![
                (eq(&peer_1.id), unordered_elements_are![eq(&interface_a)]),
                (eq(&peer_2.id), unordered_elements_are![eq(&interface_b), eq(&interface_c)]),
                (eq(&peer_leader.id), is_empty()),
            ]
        );
        Ok(())
    }

    struct Fixture {
        testee: ClusterManagerRef,
        resource_manager: ResourceManagerRef,
        peer_messaging_broker: PeerMessagingBrokerRef,
        cluster_manager_options: ClusterManagerOptions,
    }
    impl Fixture {
        async fn create() -> Fixture {
            let settings = settings::load_defaults().unwrap();

            let resource_manager = ResourceManager::new_in_memory();
            let peer_messaging_broker = PeerMessagingBroker::new(
                Arc::clone(&resource_manager),
                PeerMessagingBrokerOptions::load(&settings.config).unwrap(),
            ).await;

            let cluster_manager_options = ClusterManagerOptions::load(&settings.config).unwrap();

            let testee = ClusterManager::create(
                Arc::clone(&resource_manager),
                Arc::clone(&peer_messaging_broker),
                Vpn::Disabled,
                cluster_manager_options.clone(),
            ).await;
            Fixture {
                testee,
                resource_manager,
                peer_messaging_broker,
                cluster_manager_options,
            }
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
        let network_interface_id = NetworkInterfaceId::random();
        let interfaces = vec![
            NetworkInterfaceDescriptor {
                id: network_interface_id,
                name: NetworkInterfaceName::try_from("eth0").unwrap(),
                configuration: NetworkInterfaceConfiguration::Ethernet,
            }
        ];

        let descriptor = PeerDescriptor {
            id,
            name: PeerName::try_from(peer_name).unwrap(),
            location: PeerLocation::try_from("Ulm").ok(),
            network: PeerNetworkDescriptor {
                interfaces,
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: device,
                        name: DeviceName::try_from(format!("{peer_name}_Device_1")).unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: network_interface_id,
                        tags: vec![],
                    }
                ]
            },
            executors: ExecutorDescriptors {
                executors: vec![
                    ExecutorDescriptor {
                        id: ExecutorId::random(),
                        kind: ExecutorKind::Container {
                            engine: Engine::Docker,
                            name: ContainerName::Empty,
                            image: ContainerImage::try_from("testUrl").unwrap(),
                            volumes: vec![],
                            devices: vec![],
                            envs: vec![],
                            ports: vec![],
                            command: ContainerCommand::Default,
                            args: vec![],
                        },
                        results_url: None,
                    }
                ],
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
