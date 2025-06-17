use std::net::{IpAddr, Ipv4Addr};
use tracing::debug;
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::configuration::{parameter, PeerConfiguration};
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
        let device_interfaces = device_interfaces.into_iter()
            .map(|descriptor| parameter::DeviceInterface { descriptor });

        peer_configuration.set_all_present(device_interfaces);

        debug!("Configured network device interfaces: {:?}", peer_configuration.device_interfaces);
    }

    let ethernet_bridge = peer_descriptor.clone().network.bridge_name
        .unwrap_or(options.bridge_name_default);
    // Ethernet bridge
    {
        let bridge = parameter::EthernetBridge { name: ethernet_bridge.clone() };

        peer_configuration.set_all_present(vec![bridge]);
    }

    // GRE interfaces
    {
        peer_configuration.set_all_present(expected_gre_config_parameters);
    }

    // Joined interfaces
    // all ethernet interfaces + GRE interfaces -> bridge
    {
        let expected_joined_interfaces = expected_joined_interface_names.iter()
            .map(|name| InterfaceJoinConfig { name: name.clone(), bridge: ethernet_bridge.clone() })
            .collect::<Vec<_>>();

        peer_configuration.set_all_present(expected_joined_interfaces)
    }


    // Executors
    {
        let executors = peer_descriptor.executors.executors.into_iter()
            .map(|descriptor| parameter::Executor { descriptor });

        peer_configuration.set_all_present(executors)
    }

    Ok(())
}


pub(super) fn determine_expected_gre_interface_config_parameters(peer_id: PeerId, cluster_assignment: &ClusterAssignment) -> Result<Vec<GreInterfaceConfig>, AssignClusterError> {

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
