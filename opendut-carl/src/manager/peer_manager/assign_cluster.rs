use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use crate::manager::peer_messaging_broker::PeerMessagingBrokerRef;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;
use opendut_carl_api::proto::services::peer_messaging_broker::{downstream, ApplyPeerConfiguration};
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::configuration::{parameter, ParameterId, ParameterValue};
use opendut_types::peer::configuration::{OldPeerConfiguration, ParameterTarget, PeerConfiguration};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName, NetworkInterfaceNameError};
use tracing::debug;
use opendut_types::peer::configuration::parameter::{GreInterfaceConfig, InterfaceJoinConfig};
use crate::resource::api::resources::Resources;

pub struct AssignClusterParams {
    pub peer_messaging_broker: PeerMessagingBrokerRef,
    pub peer_id: PeerId,
    pub device_interfaces: Vec<NetworkInterfaceDescriptor>,
    pub cluster_assignment: ClusterAssignment,
    pub options: AssignClusterOptions,
}

#[derive(Clone)]
pub struct AssignClusterOptions {
    pub bridge_name_default: NetworkInterfaceName
}

#[derive(thiserror::Error, Debug)]
pub enum AssignClusterError {
    #[error("Assigning cluster for peer <{0}> failed, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("Could not assign interface name.")]
    InterfaceName { #[source] source: NetworkInterfaceNameError },
    #[error("Sending PeerConfiguration with ClusterAssignment to peer <{peer_id}> failed: {cause}")]
    SendingToPeerFailed { peer_id: PeerId, cause: String },
    #[error("Error while persisting ClusterAssignment for peer <{peer_id}>.")]
    Persistence { peer_id: PeerId, #[source] source: PersistenceError },
    #[error("IPv6 not supported for GRE interface configuration.")]
    Ipv6NotSupported,
}

fn require_ipv4_for_gre(ip_address: IpAddr) -> Result<Ipv4Addr, AssignClusterError> {
    match ip_address {
        IpAddr::V4(ip_address) => Ok(ip_address),
        IpAddr::V6(_) => Err(AssignClusterError::Ipv6NotSupported),
    }
}

fn determine_expected_gre_interface_config_parameters(peer_id: PeerId, cluster_assignment: &ClusterAssignment) -> Result<Vec<GreInterfaceConfig>, AssignClusterError> {
    let leader_remote_ip = cluster_assignment.assignments
        .iter()
        .find(|assignment| assignment.peer_id == cluster_assignment.leader)
        .map(|assignment| assignment.vpn_address)
        .map(require_ipv4_for_gre)
        .ok_or(AssignClusterError::PeerNotFound(cluster_assignment.leader))??;
    let peer_vpn_address = cluster_assignment.assignments
        .iter()
        .find(|assignment| assignment.peer_id == peer_id)
        .map(|assignment| assignment.vpn_address)
        .map(require_ipv4_for_gre)
        .ok_or(AssignClusterError::PeerNotFound(peer_id))??;

    if peer_id == cluster_assignment.leader {
        // Leader shall create GRE interfaces to bridge traffic to all peers
        let gre_configs = cluster_assignment.assignments
            .iter()
            .filter(|assignment| assignment.peer_id != cluster_assignment.leader)
            .map(|assignment| assignment.vpn_address)
            .map(require_ipv4_for_gre)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|remote_ip| GreInterfaceConfig {
                local_ip: peer_vpn_address,
                remote_ip,
            })
            .collect::<Vec<GreInterfaceConfig>>();

        Ok(gre_configs)
    } else {
        // Other peers shall only send traffic to the leader
        Ok(vec![
            GreInterfaceConfig {
                local_ip: peer_vpn_address,
                remote_ip: leader_remote_ip,
            }
        ])
    }
}

impl Resources<'_> {
    pub async fn assign_cluster(&mut self, params: AssignClusterParams) -> Result<(), AssignClusterError> {
        let AssignClusterParams { peer_messaging_broker, peer_id, cluster_assignment, device_interfaces, options } = params;

        debug!("Assigning cluster to peer <{peer_id}>.");
        let expected_gre_config_parameters = determine_expected_gre_interface_config_parameters(peer_id, &cluster_assignment)?;
        let expected_gre_interface_names = expected_gre_config_parameters
            .iter()
            .map(|config| config.interface_name()).collect::<Result<Vec<_>, _>>()
            .map_err(|source| AssignClusterError::InterfaceName { source })?;
        let expected_joined_ethernet_device_interfaces = device_interfaces
            .iter()
            .filter(|interface| matches!(interface.configuration, NetworkInterfaceConfiguration::Ethernet))
            .map(|device| device.name.clone())
            .collect::<Vec<_>>();
        let expected_joined_interface_names = expected_gre_interface_names
            .into_iter()
            .chain(expected_joined_ethernet_device_interfaces)
            .collect::<Vec<_>>();

        let (old_peer_configuration, peer_configuration) = {
            let old_peer_configuration = OldPeerConfiguration {
                cluster_assignment: Some(cluster_assignment),
            };
            self.insert(peer_id, Clone::clone(&old_peer_configuration))
                .map_err(|source| AssignClusterError::Persistence { peer_id, source })?;

            let peer_configuration = {
                let peer_descriptor = self.get::<PeerDescriptor>(peer_id)
                    .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
                    .ok_or(AssignClusterError::PeerNotFound(peer_id))?;

                let mut peer_configuration = self.get::<PeerConfiguration>(peer_id)
                    .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
                    .unwrap_or_default();

                // Network device interfaces
                {
                    let expected_network_device_ids = device_interfaces.iter().map(|device| { ParameterId(device.id.uuid) }).collect::<HashSet<ParameterId>>();
                    let absent_network_device_parameters = peer_configuration
                        .device_interfaces
                        .iter()
                        .filter(|device| !expected_network_device_ids.contains(&device.id))
                        .cloned()
                        .collect::<Vec<_>>();
                    for absent_device_interface in absent_network_device_parameters.into_iter() {
                        peer_configuration.set(absent_device_interface.value, ParameterTarget::Absent);
                    }
                    for device_interface in device_interfaces.into_iter() {
                        let device_interface = parameter::DeviceInterface { descriptor: device_interface };
                        peer_configuration.set(device_interface, ParameterTarget::Present);
                    }
                    debug!("Configured network device interfaces: {:?}", peer_configuration.device_interfaces);
                }

                let ethernet_bridge = peer_descriptor.clone().network.bridge_name
                    .unwrap_or(options.bridge_name_default);
                // Ethernet bridge
                {
                    let bridge = parameter::EthernetBridge { name: ethernet_bridge.clone() };
                    let expected_bridge_id = bridge.parameter_identifier();
                    let absent_bridge_parameters = peer_configuration
                        .ethernet_bridges
                        .iter()
                        .filter(|bridge| expected_bridge_id != bridge.id)
                        .cloned()
                        .collect::<Vec<_>>();
                    // set old bridge to absent (e.g. after renaming)
                    for absent_bridge in absent_bridge_parameters.into_iter() {
                        peer_configuration.set(absent_bridge.value, ParameterTarget::Absent);
                    }
                    // there is only one bridge definition at the moment                    
                    peer_configuration.set(bridge, ParameterTarget::Present);
                }

                // GRE interfaces
                {
                    let expected_gre_config_ids = expected_gre_config_parameters.iter().map(|gre| gre.parameter_identifier()).collect::<HashSet<_>>();
                    let absent_gre_config_parameters = peer_configuration
                        .gre_interfaces
                        .iter()
                        .filter(|gre_config| !expected_gre_config_ids.contains(&gre_config.id))
                        .cloned()
                        .collect::<Vec<_>>();
                    for absent_gre_config in absent_gre_config_parameters.into_iter() {
                        peer_configuration.set(absent_gre_config.value, ParameterTarget::Absent);
                    }
                    for gre_config in expected_gre_config_parameters.into_iter() {
                        peer_configuration.set(gre_config, ParameterTarget::Present);
                    }
                }

                // Joined interfaces
                // all ethernet interfaces + GRE interfaces -> bridge
                {
                    let expected_joined_interfaces = expected_joined_interface_names
                        .iter()
                        .map(|name| InterfaceJoinConfig { name: name.clone(), bridge: ethernet_bridge.clone() })
                        .collect::<Vec<_>>();
                    let expected_joined_interface_ids = expected_joined_interfaces
                        .iter()
                        .map(|config| config.parameter_identifier())
                        .collect::<HashSet<_>>();
                    let absent_joined_interface_parameters = peer_configuration
                        .joined_interfaces
                        .iter()
                        .filter(|joined_interface| !expected_joined_interface_ids.contains(&joined_interface.id))
                        .cloned()
                        .collect::<Vec<_>>();
                    for absent_joined_interface in absent_joined_interface_parameters.into_iter() {
                        peer_configuration.set(absent_joined_interface.value, ParameterTarget::Absent);
                    }
                    for joined_interface_config in expected_joined_interfaces.into_iter() {
                        peer_configuration.set(joined_interface_config, ParameterTarget::Present);
                    }

                }
                

                // Executors
                {
                    let expected_executor_descriptor_ids = Clone::clone(&peer_descriptor.executors).executors.iter().map(|executor| ParameterId(executor.id.uuid)).collect::<HashSet<_>>();
                    let absent_executor_descriptor_parameters = peer_configuration
                        .executors
                        .iter()
                        .filter(|executor| !expected_executor_descriptor_ids.contains(&executor.id))
                        .cloned()
                        .collect::<Vec<_>>();
                    for absent_executors in absent_executor_descriptor_parameters.into_iter() {
                        peer_configuration.set(absent_executors.value, ParameterTarget::Absent);
                    }
                    for executor_descriptor in Clone::clone(&peer_descriptor.executors).executors.into_iter() {
                        let executor = parameter::Executor { descriptor: executor_descriptor };
                        peer_configuration.set(executor, ParameterTarget::Present);
                    }
                }

                peer_configuration
            };
            // store updated peer configuration
            self.insert(peer_id, Clone::clone(&peer_configuration))
                .map_err(|source| AssignClusterError::Persistence { peer_id, source })?;

            (old_peer_configuration, peer_configuration)
        };

        peer_messaging_broker.send_to_peer(
            peer_id,
            downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                old_configuration: Some(old_peer_configuration.into()),
                configuration: Some(peer_configuration.into()),
            }),
        ).await
        .map_err(|cause| AssignClusterError::SendingToPeerFailed {
            peer_id,
            cause: cause.to_string()
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::peer_messaging_broker::{PeerMessagingBroker, PeerMessagingBrokerOptions};
    use crate::resource::manager::ResourceManager;
    use googletest::prelude::*;
    use opendut_types::cluster::{ClusterAssignment, ClusterId};
    use std::net::IpAddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use opendut_carl_api::carl::broker::stream_header;
    use crate::manager::peer_manager::tests::create_peer_descriptor;
    use crate::manager::testing::PeerFixture;

    #[tokio::test]
    async fn should_update_peer_configuration() -> anyhow::Result<()> {
        let peer = PeerFixture::new();

        let settings = crate::settings::load_defaults()?;
        let peer_id = peer.id;

        let resource_manager = ResourceManager::new_in_memory();
        let peer_messaging_broker = PeerMessagingBroker::new(
            Arc::clone(&resource_manager),
            PeerMessagingBrokerOptions::load(&settings.config).unwrap(),
        ).await;

        let old_peer_configuration = OldPeerConfiguration {
            cluster_assignment: None,
        };
        let peer_configuration = PeerConfiguration::default();
        resource_manager.resources_mut(async |resources| {
            resources.insert(peer_id, create_peer_descriptor())?;
            resources.insert(peer_id, Clone::clone(&old_peer_configuration))?;
            resources.insert(peer_id, Clone::clone(&peer_configuration))
        }).await??;

        let (_, mut receiver) = peer_messaging_broker.open(peer_id, IpAddr::from_str("1.2.3.4")?, stream_header::ExtraHeaders::default()).await?;
        let received = receiver.recv().await.unwrap()
            .message.unwrap();
        assert_that!(
            received,
            eq(&downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                old_configuration: Some(Clone::clone(&old_peer_configuration).into()),
                configuration: Some(Clone::clone(&peer_configuration).into()),
            }))
        );


        let cluster_assignment = ClusterAssignment {
            id: ClusterId::random(),
            leader: PeerId::random(),
            assignments: vec![],
        };


        resource_manager.resources_mut(async |resources|
            resources.assign_cluster(AssignClusterParams {
                peer_messaging_broker: Arc::clone(&peer_messaging_broker),
                peer_id,
                cluster_assignment: Clone::clone(&cluster_assignment),
                device_interfaces: vec![],
                options: AssignClusterOptions {
                    bridge_name_default: NetworkInterfaceName::try_from("br-opendut").unwrap(),
                }
            }).await
        ).await??;


        let old_peer_configuration = OldPeerConfiguration {
            cluster_assignment: Some(cluster_assignment),
        };
        assert_that!(
            resource_manager.get::<OldPeerConfiguration>(peer_id).await?.as_ref(),
            some(eq(&old_peer_configuration))
        );

        let mut peer_configuration = PeerConfiguration::default();
        peer_configuration.set(
            parameter::EthernetBridge { name: NetworkInterfaceName::try_from("br-opendut-1")? },
            ParameterTarget::Present
        );

        let received = receiver.recv().await.unwrap()
            .message.unwrap();

        let downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
            old_configuration, configuration
        }) = received else { panic!() };

        assert_that!(OldPeerConfiguration::try_from(old_configuration.unwrap())?, eq(&old_peer_configuration));
        assert_that!(PeerConfiguration::try_from(configuration.unwrap())?, eq(&peer_configuration));

        Ok(())
    }

}
