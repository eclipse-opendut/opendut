use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::PeerId;
use opendut_types::util::net::NetworkInterfaceName;
use crate::service::network_interface::can;
use crate::service::network_interface::gre;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub async fn handle(
    cluster_assignment: ClusterAssignment,
    self_id: PeerId,
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
) -> Result<(), Error> {

    let local_ip = determine_local_ip(&cluster_assignment, self_id)?;

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

    //TODO join device interfaces to bridge

    setup_can(&cluster_assignment, self_id, network_interface_manager).await?;

    Ok(())
}

pub async fn setup_can(
    cluster_assignment: &ClusterAssignment,
    self_id: PeerId,
    network_interface_manager: NetworkInterfaceManagerRef
) -> Result<(), Error> {

    let can_bridge_name = crate::common::default_can_bridge_name();
    let own_can_interfaces = get_own_can_interfaces(&cluster_assignment, self_id)?;
    can::setup_local_routing(
        &can_bridge_name, 
        own_can_interfaces,
        Arc::clone(&network_interface_manager)
    ).await
    .map_err(Error::LocalCanRoutingSetupFailed)?;

    let local_ip = determine_local_ip(&cluster_assignment, self_id)?;
    let is_leader = cluster_assignment.leader == self_id;

    if is_leader {

        let remote_ips = determine_remote_ips(&cluster_assignment, self_id, &local_ip)?;
        can::setup_remote_routing_server(
            &can_bridge_name, 
            &remote_ips
        ).await
        .map_err(Error::RemoteCanRoutingSetupFailed)?;

    } else {
        
        let leader_ip = determine_leader_ip(&cluster_assignment)?;
        can::setup_remote_routing_client(
            &can_bridge_name, 
            &local_ip, 
            &leader_ip
        ).await
        .map_err(Error::RemoteCanRoutingSetupFailed)?;
    }

    Ok(())
}

fn determine_local_ip(cluster_assignment: &ClusterAssignment, self_id: PeerId) -> Result<IpAddr, Error> {

    let local_ip = cluster_assignment.assignments.iter().find_map(|assignment| {
        let is_local = assignment.peer_id == self_id;

        is_local
            .then_some(&assignment.vpn_address)
    }).ok_or(Error::IpAddressNotDeterminable { kind: IpErrorKind::Local })?;

    Ok(local_ip.to_owned())
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
        let leader_ip = determine_leader_ip(cluster_assignment)?;

        vec![leader_ip]
    };

    Ok(remote_ips)
}

fn determine_leader_ip(cluster_assignment: &ClusterAssignment) -> Result<IpAddr, Error>{
    let leader_ip = cluster_assignment.assignments.iter().find_map(|peer_assignment| {
        let is_leader = peer_assignment.peer_id == cluster_assignment.leader;

        is_leader
            .then_some(peer_assignment.vpn_address)
    }).ok_or(Error::IpAddressNotDeterminable { kind: IpErrorKind::Leader })?;

    Ok(leader_ip)
}

fn require_ipv4_for_gre(ip_address: IpAddr) -> Result<Ipv4Addr, Error> {
    match ip_address {
        IpAddr::V4(ip_address) => Ok(ip_address),
        IpAddr::V6(_) => Err(Error::Ipv6NotSupported),
    }
}

fn get_own_can_interfaces(
    cluster_assignment: &ClusterAssignment,
    self_id: PeerId) -> Result<Vec<NetworkInterfaceName>, Error>{

    let own_cluster_assignment = cluster_assignment.assignments.iter().find(|assignment| assignment.peer_id == self_id).unwrap();

    // TODO: Use some proper way to determine whether an interface is a CAN one or not
    let own_can_interfaces: Vec<NetworkInterfaceName> = own_cluster_assignment.device_interfaces.iter()
        .filter(|interface| interface.name().contains("can"))
        .cloned()
        .collect();

    Ok(own_can_interfaces)
    }

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not determine {kind:?} IP address from ClusterAssignment.")]
    IpAddressNotDeterminable { kind: IpErrorKind },
    #[error("IPv6 isn't yet supported for GRE interfaces.")]
    Ipv6NotSupported,
    #[error("GRE interface setup failed: {0}")]
    GreInterfaceSetupFailed(gre::Error),
    #[error("Local CAN routing setup failed: {0}")]
    LocalCanRoutingSetupFailed(can::Error),
    #[error("Remote CAN routing setup failed: {0}")]
    RemoteCanRoutingSetupFailed(can::Error),
}

#[derive(Debug)]
pub enum IpErrorKind {
    Local,
    Leader,
}
