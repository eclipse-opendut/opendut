use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::PeerId;
use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface;
use crate::service::network_interface::{bridge, gre};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub async fn network_interfaces_setup(
    cluster_assignment: ClusterAssignment,
    self_id: PeerId,
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
) -> Result<(), Error> {

    bridge::recreate(bridge_name, Arc::clone(&network_interface_manager)).await
        .map_err(Error::BridgeRecreationFailed)?;

    let local_peer_assignment = cluster_assignment.assignments.iter().find(|assignment| {
        assignment.peer_id == self_id
    }).ok_or(Error::LocalPeerAssignmentNotFound { self_id })?;

    let local_ip = local_peer_assignment.vpn_address;

    let remote_ips = determine_remote_ips(&cluster_assignment, self_id, &local_ip)?;

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


    join_device_interfaces_to_bridge(&local_peer_assignment.device_interfaces, bridge_name, Arc::clone(&network_interface_manager)).await
        .map_err(Error::JoinDeviceInterfaceToBridgeFailed)?;

    Ok(())
}

fn determine_remote_ips(cluster_assignment: &ClusterAssignment, self_id: PeerId, local_ip: &IpAddr) -> Result<Vec<IpAddr>, Error> {
    let is_leader = cluster_assignment.leader == self_id;

    let remote_ips = if is_leader {
        cluster_assignment.assignments.iter()
            .map(|assignment| assignment.vpn_address)
            .filter(|address| address != local_ip)
            .collect()
    }
    else {
        let leader_ip = cluster_assignment.assignments.iter().find_map(|peer_assignment| {
            let is_leader = peer_assignment.peer_id == cluster_assignment.leader;

            is_leader
                .then_some(peer_assignment.vpn_address)
        }).ok_or(Error::LeaderIpAddressNotDeterminable)?;

        vec![leader_ip]
    };

    Ok(remote_ips)
}

fn require_ipv4_for_gre(ip_address: IpAddr) -> Result<Ipv4Addr, Error> {
    match ip_address {
        IpAddr::V4(ip_address) => Ok(ip_address),
        IpAddr::V6(_) => Err(Error::Ipv6NotSupported),
    }
}

async fn join_device_interfaces_to_bridge(
    device_interfaces: &Vec<NetworkInterfaceName>,
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef
) -> Result<(), network_interface::manager::Error> {
    let bridge = network_interface_manager.try_find_interface(bridge_name).await?;

    for interface in device_interfaces {
        let interface = network_interface_manager.try_find_interface(interface).await?;
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
    #[error("Could not determine leader IP address from ClusterAssignment.")]
    LeaderIpAddressNotDeterminable,
    #[error("IPv6 isn't yet supported for GRE interfaces.")]
    Ipv6NotSupported,
    #[error("GRE interface setup failed: {0}")]
    GreInterfaceSetupFailed(gre::Error),
    #[error("Joining device interface to bridge failed: {0}")]
    JoinDeviceInterfaceToBridgeFailed(network_interface::manager::Error),
}
