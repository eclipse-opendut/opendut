use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub async fn create(
    bridge_name: &NetworkInterfaceName,
    network_interface_manager: NetworkInterfaceManagerRef,
) -> Result<(), network_interface::manager::Error> {

    let bridge = network_interface_manager.create_empty_bridge(bridge_name).await?;
    network_interface_manager.set_interface_up(&bridge).await?;

    Ok(())
}
