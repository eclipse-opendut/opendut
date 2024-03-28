use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use opendut_types::cluster::{ClusterAssignment, PeerClusterAssignment};
use opendut_types::peer::PeerId;
use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

use crate::service::network_interface;
use crate::service::network_interface::{bridge, gre};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::service::can_manager::CanManagerRef;

#[tracing::instrument(skip(cluster_assignment, can_manager, network_interface_manager), level="trace")]
pub async fn network_interfaces_setup(
    cluster_assignment: ClusterAssignment,
    self_id: PeerId,
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
    can_manager: CanManagerRef,
) -> Result<(), Error> {

    bridge::recreate(bridge_name, Arc::clone(&network_interface_manager)).await
        .map_err(Error::BridgeRecreationFailed)?;

    let local_peer_assignment = cluster_assignment.assignments.iter().find(|assignment| {
        assignment.peer_id == self_id
    }).ok_or(Error::LocalPeerAssignmentNotFound { self_id })?;

    let local_ip = local_peer_assignment.vpn_address;

    let remote_ips = determine_remote_ips(&cluster_assignment, self_id)?;

    let local_ip = require_ipv4_for_gre(local_ip)?;
    let remote_ips = remote_ips.into_iter()
        .map(require_ipv4_for_gre)
        .collect::<Result<Vec<_>, _>>()?;

    gre::setup_interfaces(
        &local_ip,
        &remote_ips,
        bridge_name,
        Arc::clone(&network_interface_manager),
    ).await
    .map_err(Error::GreInterfaceSetupFailed)?;

    let own_ethernet_interfaces = get_own_ethernet_interfaces(&cluster_assignment, self_id)?;
    join_device_interfaces_to_bridge(&own_ethernet_interfaces, bridge_name, Arc::clone(&network_interface_manager)).await
        .map_err(Error::JoinDeviceInterfaceToBridgeFailed)?;

    setup_can(&cluster_assignment, self_id, can_manager).await?;

    Ok(())
}

pub async fn setup_can(
    cluster_assignment: &ClusterAssignment,
    self_id: PeerId,
    can_manager: CanManagerRef
) -> Result<(), Error> {

    let can_bridge_name = crate::common::default_can_bridge_name();
    let own_can_interfaces = get_own_can_interfaces(cluster_assignment, self_id)?;
    can_manager.setup_local_routing(
        &can_bridge_name, 
        own_can_interfaces,
    ).await
    .map_err(Error::LocalCanRoutingSetupFailed)?;

    let local_peer_assignment = cluster_assignment.assignments.iter().find(|assignment| {
        assignment.peer_id == self_id
    }).ok_or(Error::LocalPeerAssignmentNotFound { self_id })?;

    let is_leader = cluster_assignment.leader == self_id;

    let server_port = local_peer_assignment.can_server_port;

    if is_leader {

        let remote_assignments = determine_remote_assignments(cluster_assignment, self_id)?;
        can_manager.setup_remote_routing_server(
            &can_bridge_name, 
            &remote_assignments
        ).await
        .map_err(Error::RemoteCanRoutingSetupFailed)?;

    } else {        

        let leader_assignment = determine_leader_assignment(cluster_assignment)?;
        can_manager.setup_remote_routing_client(
            &can_bridge_name, 
            &leader_assignment.vpn_address,
            &server_port
        ).await
        .map_err(Error::RemoteCanRoutingSetupFailed)?;
    }

    Ok(())
}

fn determine_remote_ips(cluster_assignment: &ClusterAssignment, self_id: PeerId) -> Result<Vec<IpAddr>, Error> {
    let remote_assignments = determine_remote_assignments(cluster_assignment, self_id);
    let remote_ips = remote_assignments?.iter().map(|remote_assignment| remote_assignment.vpn_address).collect();

    Ok(remote_ips)
}

fn determine_remote_assignments(cluster_assignment: &ClusterAssignment, self_id: PeerId) -> Result<Vec<PeerClusterAssignment>, Error> {
    let is_leader = cluster_assignment.leader == self_id;

    let remote_peer_cluster_assignments = if is_leader {
        cluster_assignment.assignments.iter()
            .filter(|assignment| assignment.peer_id != self_id).cloned()
            .collect::<Vec<PeerClusterAssignment>>()
    }
    else {
        let leader_ip = determine_leader_assignment(cluster_assignment)?;

        vec![leader_ip.clone()]
    };

    Ok(remote_peer_cluster_assignments)
}

fn determine_leader_assignment(cluster_assignment: &ClusterAssignment) -> Result<&PeerClusterAssignment, Error>{
    let leader_assignment = cluster_assignment.assignments
        .iter().find(|peer_assignment| 
            peer_assignment.peer_id == cluster_assignment.leader
            ).ok_or(Error::LeaderNotDeterminable)?;

    Ok(leader_assignment)
}

fn require_ipv4_for_gre(ip_address: IpAddr) -> Result<Ipv4Addr, Error> {
    match ip_address {
        IpAddr::V4(ip_address) => Ok(ip_address),
        IpAddr::V6(_) => Err(Error::Ipv6NotSupported),
    }
}

fn get_own_ethernet_interfaces(cluster_assignment: &ClusterAssignment,
    self_id: PeerId) -> Result<Vec<NetworkInterfaceDescriptor>, Error> {

    let own_cluster_assignment = cluster_assignment.assignments.iter().find(|assignment| assignment.peer_id == self_id).unwrap();

    let own_ethernet_interfaces: Vec<NetworkInterfaceDescriptor> = own_cluster_assignment.device_interfaces.iter()
        .filter(|interface| interface.configuration == NetworkInterfaceConfiguration::Ethernet)
        .cloned()
        .collect();

    Ok(own_ethernet_interfaces)
}

fn get_own_can_interfaces(
    cluster_assignment: &ClusterAssignment,
    self_id: PeerId) -> Result<Vec<NetworkInterfaceDescriptor>, Error>{

    let own_cluster_assignment = cluster_assignment.assignments.iter().find(|assignment| assignment.peer_id == self_id).unwrap();

    let own_can_interfaces: Vec<NetworkInterfaceDescriptor> = own_cluster_assignment.device_interfaces.iter()
        .filter(|interface| matches!(interface.configuration, NetworkInterfaceConfiguration::Can{ .. }))
        .cloned()
        .collect();

    Ok(own_can_interfaces)
    }

async fn join_device_interfaces_to_bridge(
    device_interfaces: &Vec<NetworkInterfaceDescriptor>,
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef
) -> Result<(), network_interface::manager::Error> {
    let bridge = network_interface_manager.try_find_interface(bridge_name).await?;

    for interface in device_interfaces {
        let interface = network_interface_manager.try_find_interface(&interface.name).await?;
        network_interface_manager.join_interface_to_bridge(&interface, &bridge).await?;
        log::debug!("Joined device interface {interface} to bridge {bridge}.");
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("(Re-)Creating the bridge failed: {0}")]
    BridgeRecreationFailed(network_interface::manager::Error),
    #[error("Could not find PeerAssignment for this peer (<{self_id}>) in the ClusterAssignment.")]
    LocalPeerAssignmentNotFound { self_id: PeerId },
    #[error("Could not determine leader from ClusterAssignment.")]
    LeaderNotDeterminable,
    #[error("IPv6 isn't yet supported for GRE interfaces.")]
    Ipv6NotSupported,
    #[error("GRE interface setup failed: {0}")]
    GreInterfaceSetupFailed(gre::Error),
    #[error("Local CAN routing setup failed: {0}")]
    LocalCanRoutingSetupFailed(crate::service::can_manager::Error),
    #[error("Remote CAN routing setup failed: {0}")]
    RemoteCanRoutingSetupFailed(crate::service::can_manager::Error),
    #[error("Joining device interface to bridge failed: {0}")]
    JoinDeviceInterfaceToBridgeFailed(network_interface::manager::Error),
}
