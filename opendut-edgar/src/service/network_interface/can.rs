use std::net::IpAddr;
use tokio::process::Command;

use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while managing CAN interfaces: {0}")]
    NetworkInterfaceError(#[from] network_interface::manager::Error),
    #[error("Failure while invoking command line program '{command}': {cause}")]
    CommandLineProgramExecution { command: String, cause: std::io::Error },
    #[error("{message}")]
    Other { message: String },
}

pub async fn setup_local_routing(
    bridge_name: &NetworkInterfaceName,
    local_can_interfaces: Vec<NetworkInterfaceName>,
    network_interface_manager: NetworkInterfaceManagerRef
) -> Result<(), Error> {


    create_can_bridge(bridge_name, &network_interface_manager).await
        .map_err(|cause| Error::Other { message: format!("Error while creating CAN bridge: {cause}") })?;

    for interface in local_can_interfaces {
        network_interface_manager.create_can_route(bridge_name, &interface, true, 2).await?;
        network_interface_manager.create_can_route(bridge_name, &interface, false, 2).await?;
        network_interface_manager.create_can_route(&interface, bridge_name, true, 2).await?;
        network_interface_manager.create_can_route(&interface, bridge_name, false, 2).await?;
    }

    Ok(())
}

async fn create_can_bridge(bridge_name: &NetworkInterfaceName, network_interface_manager: &NetworkInterfaceManagerRef) -> anyhow::Result<()> {

    if network_interface_manager.find_interface(bridge_name).await?.is_none() {
        log::debug!("Creating CAN bridge '{bridge_name}'.");
        let bridge = network_interface_manager.create_vcan_interface(bridge_name).await?;
        network_interface_manager.set_interface_up(&bridge).await?;
    } else {
        log::debug!("Not creating CAN bridge '{bridge_name}', because it already exists.");
    }

    Ok(())
}

fn peer_ip_to_leader_port(peer_ip: &IpAddr) -> anyhow::Result<u16>{
    assert!(peer_ip.is_ipv4());
    let ip_bytes: Vec<u8> = peer_ip.to_string().split(".").map(|b| b.parse::<u8>().unwrap()).collect();
    let port = ((ip_bytes[2] as u16) << 8) | ip_bytes[3] as u16;
    Ok(port)
}

pub async fn setup_remote_routing_client(bridge_name: &NetworkInterfaceName, local_ip: &IpAddr, leader_ip: &IpAddr) -> Result<(), Error> {

    let leader_port = peer_ip_to_leader_port(local_ip).unwrap();

    println!("spawning cannelloni client process");

    // TODO: We should do some monitoring of the cannelloni process to restart if needed. Should maybe also redirect its stdout and stderr
    // TODO: We need some more configuration options here, like the aggregation timeout
    let child = Command::new("cannelloni")
        .arg("-I")
        .arg(bridge_name.name())
        .arg("-S")
        .arg("c")
        .arg("-r")
        .arg(leader_port.to_string())
        .arg("-R")
        .arg(leader_ip.to_string())
        .spawn()
        .map_err(|cause| Error::CommandLineProgramExecution { command: "cannelloni".to_string(), cause })?;

    Ok(())
}

pub async fn setup_remote_routing_server(bridge_name: &NetworkInterfaceName, remote_ips: &Vec<IpAddr>) -> Result<(), Error>  {


    for remote_ip in remote_ips {
        let leader_port = peer_ip_to_leader_port(&remote_ip).unwrap();
        println!("spawning cannelloni server process");

        let child = Command::new("cannelloni")
            .arg("-I")
            .arg(bridge_name.name())
            .arg("-S")
            .arg("s")
            .arg("-l")
            .arg(leader_port.to_string())
            .arg("-R")
            .arg(remote_ip.to_string())
            .spawn()
            .map_err(|cause| Error::CommandLineProgramExecution { command: "cannelloni".to_string(), cause })?;
    }

    Ok(())
}