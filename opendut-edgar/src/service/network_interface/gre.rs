use std::net::Ipv4Addr;
use std::sync::Arc;

use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

const GRE_INTERFACE_NAME_PREFIX: &str = "gre-opendut";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while managing network interfaces: {0}")]
    NetworkInterfaceError(#[from] network_interface::manager::Error),
    #[error("{message}")]
    Other { message: String },
}

pub async fn setup_interfaces(
    local_ip: &Ipv4Addr,
    remote_ips: &[Ipv4Addr],
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
) -> Result<(), Error> {

    remove_existing_interfaces(Arc::clone(&network_interface_manager)).await?;

    for (interface_index, remote_ip) in remote_ips.iter().enumerate() {
        create_interface(local_ip, remote_ip, interface_index, bridge_name, Arc::clone(&network_interface_manager)).await?;
    }

    Ok(())
}

async fn remove_existing_interfaces(network_interface_manager: NetworkInterfaceManagerRef) -> Result<(), Error> {

    let interfaces_to_remove = network_interface_manager.list_interfaces().await?
        .into_iter()
        .filter(|interface| interface.name.name().starts_with(GRE_INTERFACE_NAME_PREFIX));

    for interface in interfaces_to_remove {
        network_interface_manager.delete_interface(&interface).await?;
    }

    Ok(())
}

async fn create_interface(
    local_ip: &Ipv4Addr,
    remote_ip: &Ipv4Addr,
    interface_index: usize,
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
) -> Result<(), Error> {

    let interface_name = NetworkInterfaceName::try_from(format!("{}{}", GRE_INTERFACE_NAME_PREFIX, interface_index))
        .map_err(|cause| Error::Other { message: format!("Error while constructing GRE interface name: {cause}") })?;

    let gre_interface = network_interface_manager.create_gretap_v4_interface(&interface_name, local_ip, remote_ip).await?;
    log::trace!("Created GRE interface '{gre_interface}'.");
    network_interface_manager.set_interface_up(&gre_interface).await?;
    log::trace!("Set GRE interface '{interface_name}' to 'up'.");

    let bridge = network_interface_manager.try_find_interface(bridge_name).await?;
    network_interface_manager.join_interface_to_bridge(&gre_interface, &bridge).await?;

    Ok(())
}
