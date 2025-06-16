use std::collections::HashSet;
use std::net::{IpAddr, Ipv4Addr};
use tracing::debug;
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::configuration::{parameter, ParameterId, ParameterTarget, ParameterValue, PeerConfiguration};
use opendut_types::peer::configuration::parameter::{GreInterfaceConfig, InterfaceJoinConfig};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor};
use crate::manager::peer_manager::{AssignClusterError, AssignClusterOptions};


pub(super) fn update_peer_configuration(
    peer_configuration: &mut PeerConfiguration,
    peer_descriptor: PeerDescriptor,
    expected_gre_config_parameters: Vec<GreInterfaceConfig>,
    device_interfaces: Vec<NetworkInterfaceDescriptor>,
    options: AssignClusterOptions,
) -> Result<(), AssignClusterError> {

    let expected_joined_interface_names = {
        let expected_gre_interface_names = expected_gre_config_parameters
            .iter()
            .map(|config| config.interface_name()).collect::<Result<Vec<_>, _>>()
            .map_err(|source| AssignClusterError::InterfaceName { source })?;

        let expected_joined_ethernet_device_interfaces = device_interfaces
            .iter()
            .filter(|interface| matches!(interface.configuration, NetworkInterfaceConfiguration::Ethernet))
            .map(|device| device.name.clone())
            .collect::<Vec<_>>();

         expected_gre_interface_names
            .into_iter()
            .chain(expected_joined_ethernet_device_interfaces)
            .collect::<Vec<_>>()
    };

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
        let expected_executor_descriptor_ids = Clone::clone(&peer_descriptor.executors).executors.iter()
            .map(|executor| ParameterId(executor.id.uuid)) //TODO this can't be good
            .collect::<HashSet<_>>();

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


    // RemotePeerConnectionCheck
    // { //FIXME
    //     let remote_peers = vec![]; //TODO
    //
    //     let expected_present_parameters = remote_peers.into_iter()
    //         .map(|(remote_peer_id, remote_ip)| {
    //             parameter::RemotePeerConnectionCheck { remote_peer_id, remote_ip }
    //         })
    //         .collect::<Vec<_>>();
    //
    //     for connection_check in peer_configuration.remote_peer_connection_checks {
    //         if expected_present_parameters.contains(&connection_check).not() {
    //             peer_configuration.set(connection_check.value, ParameterTarget::Absent)
    //         }
    //     }
    //
    //
    //
    //     // let bridge = parameter::EthernetBridge { name: ethernet_bridge.clone() };
    //     // let expected_bridge_id = bridge.parameter_identifier();
    //     // let absent_bridge_parameters = peer_configuration
    //     //     .ethernet_bridges
    //     //     .iter()
    //     //     .filter(|bridge| expected_bridge_id != bridge.id)
    //     //     .cloned()
    //     //     .collect::<Vec<_>>();
    //     // // set old bridge to absent (e.g. after renaming)
    //     // for absent_bridge in absent_bridge_parameters.into_iter() {
    //     //     peer_configuration.set(absent_bridge.value, ParameterTarget::Absent);
    //     // }
    //     // // there is only one bridge definition at the moment
    //     // peer_configuration.set(bridge, ParameterTarget::Present);
    // }

    Ok(())
}


pub(super) fn determine_expected_gre_interface_config_parameters(peer_id: PeerId, cluster_assignment: &ClusterAssignment) -> Result<Vec<GreInterfaceConfig>, AssignClusterError> {
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

fn require_ipv4_for_gre(ip_address: IpAddr) -> Result<Ipv4Addr, AssignClusterError> {
    match ip_address {
        IpAddr::V4(ip_address) => Ok(ip_address),
        IpAddr::V6(_) => Err(AssignClusterError::Ipv6NotSupported),
    }
}
