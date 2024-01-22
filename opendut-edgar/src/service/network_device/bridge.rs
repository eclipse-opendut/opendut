use std::rc::Rc;

use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_device;
use crate::service::network_device::manager::NetworkDeviceManager;

pub async fn create(
    bridge_name: &NetworkInterfaceName,
    network_device_manager: &Rc<NetworkDeviceManager>
) -> Result<(), network_device::manager::Error> {

    let bridge = network_device_manager.create_empty_bridge(bridge_name).await?;
    network_device_manager.set_interface_up(&bridge).await?;

    Ok(())
}
