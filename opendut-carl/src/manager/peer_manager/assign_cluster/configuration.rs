use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use tracing::debug;
use opendut_model::cluster::ClusterAssignment;
use opendut_model::peer::configuration::{parameter, PeerConfiguration};
use opendut_model::peer::configuration::parameter::{GreInterfaceConfig, InterfaceJoinConfig};
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};
use crate::manager::peer_manager::{AssignClusterError, AssignClusterOptions};


pub(super) fn update_peer_configuration(
    peer_configuration: &mut PeerConfiguration,
    peer_descriptor: PeerDescriptor,
    cluster_assignment: &ClusterAssignment,
    expected_device_interfaces: Vec<NetworkInterfaceDescriptor>,
    options: AssignClusterOptions,
) -> Result<(), AssignClusterError> {

    let expected_gre_config_parameters = determine_expected_gre_interface_config_parameters(peer_descriptor.id, cluster_assignment)?;

    let expected_joined_interface_names = {
        let expected_gre_interface_names =
            expected_gre_config_parameters.iter()
                .map(GreInterfaceConfig::interface_name).collect::<Result<Vec<_>, _>>()
                .map_err(|source| AssignClusterError::InterfaceName { source })?;

        let expected_joined_ethernet_device_interfaces =
            expected_device_interfaces.iter()
                .filter(|interface| matches!(interface.configuration, NetworkInterfaceConfiguration::Ethernet))
                .map(|device| device.name.clone())
                .collect::<Vec<_>>();

         expected_gre_interface_names.into_iter()
            .chain(expected_joined_ethernet_device_interfaces)
            .collect::<Vec<_>>()
    };

    let PeerConfiguration {
        device_interfaces,
        ethernet_bridges,
        executors,
        gre_interfaces,
        joined_interfaces,
        remote_peer_connection_checks,
        can_connections,
        can_bridges,
        can_local_routes,
    } = peer_configuration;

    let can_device_names = expected_device_interfaces.iter()
        .filter(|descriptor| matches!(descriptor.configuration, NetworkInterfaceConfiguration::Can { .. }))
        .map(|device_interfaces| device_interfaces.name.clone())
        .collect::<Vec<_>>();
    let can_enabled = !can_device_names.is_empty();
    let device_dependencies = { // Network device interfaces
        let expected_device_interfaces = expected_device_interfaces.into_iter()
            .map(|descriptor| parameter::DeviceInterface { descriptor });

        let ids = device_interfaces.set_all_present(expected_device_interfaces, vec![]);

        debug!("Configured network device interfaces: {:?}", device_interfaces);
        ids
    };

    if can_enabled {
        // CAN bridge
        let can_bridge = NetworkInterfaceName::from_str("br-vcan-opendut").unwrap();
        let expected_can_bridges = vec![parameter::CanBridge {
            name: can_bridge.clone(),
        }];
        let can_dependencies = can_bridges.set_all_present(expected_can_bridges, device_dependencies.clone());

        // CAN connections
        if cluster_assignment.leader == peer_descriptor.id {
            let remote_peers = cluster_assignment.non_leader_assignments();
            let expected_can_connections = remote_peers.into_iter()
                .map(|(remote_peer_id, peer_cluster_assignment)| {
                    parameter::CanConnection {
                        can_interface_name: can_bridge.clone(),
                        local_is_server: true,
                        remote_peer_id,
                        remote_ip: peer_cluster_assignment.vpn_address,
                        remote_port: peer_cluster_assignment.can_server_port,
                        local_port: peer_cluster_assignment.can_server_port,
                        buffer_timeout_microseconds: 100,
                    }
                });
            can_connections.set_all_present(expected_can_connections, can_dependencies.clone());
        } else {
            let leader = cluster_assignment.leader_assignment();
            match leader {
                Some(leader_assignment) => {
                    let can_connection = parameter::CanConnection {
                        can_interface_name: can_bridge.clone(),
                        local_is_server: false,
                        remote_peer_id: cluster_assignment.leader,
                        remote_ip: leader_assignment.vpn_address,
                        remote_port: leader_assignment.can_server_port,
                        local_port: leader_assignment.can_server_port,
                        buffer_timeout_microseconds: 100,
                    };
                    let expected_can_connections = vec![can_connection];
                    can_connections.set_all_present(expected_can_connections, can_dependencies.clone());
                }
                None => {
                    return Err(AssignClusterError::PeerNotFound(cluster_assignment.leader));
                }
            }
        }

        // CAN local routes
        let mut expected_can_local_routes = vec![];
        for can_device in &can_device_names {
            expected_can_local_routes.push(parameter::CanLocalRoute {
                can_source_device_name: can_device.clone(),
                can_destination_device_name: can_bridge.clone(),
            });
            expected_can_local_routes.push(parameter::CanLocalRoute {
                can_source_device_name: can_bridge.clone(),
                can_destination_device_name: can_device.clone(),
            });
        }
        can_local_routes.set_all_present(expected_can_local_routes, can_dependencies.clone());

    } else {
        // Clear CAN-related configurations if no CAN interfaces are expected
        can_connections.set_all_absent();
        can_bridges.set_all_absent();
        can_local_routes.set_all_absent();
    }

    let ethernet_bridge = peer_descriptor.network.bridge_name
        .unwrap_or(options.bridge_name_default);

    let bridge_dependencies = { // Ethernet bridge
        let bridge = parameter::EthernetBridge { name: ethernet_bridge.clone() };

        ethernet_bridges.set_all_present(vec![bridge], vec![])
    };

    // GRE interfaces
    let gre_upstream_dependencies = device_dependencies.into_iter().chain(bridge_dependencies).collect::<Vec<_>>();
    let gre_dependencies = gre_interfaces.set_all_present(expected_gre_config_parameters, gre_upstream_dependencies);

    
    { // Joined interfaces (all ethernet interfaces + GRE interfaces -> bridge)
        let expected_joined_interfaces = expected_joined_interface_names.iter()
            .map(|name| InterfaceJoinConfig { name: name.clone(), bridge: ethernet_bridge.clone() })
            .collect::<Vec<_>>();

        joined_interfaces.set_all_present(expected_joined_interfaces, gre_dependencies);
    }

    { // Executors
        let expected_executors = peer_descriptor.executors.executors.into_iter()
            .map(|descriptor| parameter::Executor { descriptor });

        executors.set_all_present(expected_executors, vec![]);
    }

    { //Remote Peer Connection Checks
        if cluster_assignment.leader == peer_descriptor.id {
            let remote_peers = cluster_assignment.non_leader_assignments();

            let expected_remote_peer_connection_checks = remote_peers.into_iter()
                .map(|(remote_peer_id, peer_cluster_assignment)| {
                    parameter::RemotePeerConnectionCheck {
                        remote_peer_id,
                        remote_ip: peer_cluster_assignment.vpn_address,
                    }
                })
                .collect::<Vec<_>>();

            remote_peer_connection_checks.set_all_present(expected_remote_peer_connection_checks, vec![]);
        } else {
            // Clear other peers, in case the Leader has changed
            remote_peer_connection_checks.set_all_absent();
        }
    }

    Ok(())
}


fn determine_expected_gre_interface_config_parameters(peer_id: PeerId, cluster_assignment: &ClusterAssignment) -> Result<Vec<GreInterfaceConfig>, AssignClusterError> {
    let leader_remote_ip = cluster_assignment.leader_assignment()
        .map(|assignment| assignment.vpn_address)
        .map(require_ipv4_for_gre)
        .ok_or(AssignClusterError::PeerNotFound(cluster_assignment.leader))??;

    if peer_id == cluster_assignment.leader {
        // Leader shall create GRE interfaces to bridge traffic to all peers
        let remote_peer_assignments = cluster_assignment.non_leader_assignments();

        let gre_configs = remote_peer_assignments.values()
            .map(|assignment| assignment.vpn_address)
            .map(require_ipv4_for_gre)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|remote_ip| GreInterfaceConfig {
                local_ip: leader_remote_ip,
                remote_ip,
            })
            .collect::<Vec<GreInterfaceConfig>>();

        Ok(gre_configs)
    } else {
        // Other peers shall only send traffic to the leader
        let peer_vpn_address = cluster_assignment.assignments
            .get(&peer_id)
            .map(|assignment| assignment.vpn_address)
            .map(require_ipv4_for_gre)
            .ok_or(AssignClusterError::PeerNotFound(peer_id))??;

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
