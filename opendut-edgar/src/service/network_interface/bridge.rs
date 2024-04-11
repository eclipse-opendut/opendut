use tracing::debug;
use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub(crate) async fn create(
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
) -> Result<(), network_interface::manager::Error> {

    let bridge = network_interface_manager.create_empty_bridge(bridge_name).await?;
    network_interface_manager.set_interface_up(&bridge).await?;

    Ok(())
}

pub(crate) async fn recreate(bridge_name: &NetworkInterfaceName, network_interface_manager: NetworkInterfaceManagerRef) -> Result<(), network_interface::manager::Error> {

    if let Some(existing_bridge) = network_interface_manager.find_interface(bridge_name).await? {
        debug!("Deleting existing bridge '{bridge_name}' before recreating it, to clear any previous joins.");
        network_interface_manager.delete_interface(&existing_bridge).await?;
    }

    debug!("Creating bridge '{bridge_name}'.");
    create(
        bridge_name,
        network_interface_manager,
    ).await?;

    Ok(())
}
