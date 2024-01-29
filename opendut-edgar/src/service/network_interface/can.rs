use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error while managing CAN interfaces: {0}")]
    NetworkInterfaceError(#[from] network_interface::manager::Error),
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
        network_interface_manager.create_can_route(bridge_name, &interface, true).await?;
        network_interface_manager.create_can_route(bridge_name, &interface, false).await?;
        network_interface_manager.create_can_route(&interface, bridge_name, true).await?;
        network_interface_manager.create_can_route(&interface, bridge_name, false).await?;
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

pub async fn setup_remote_routing() {

}