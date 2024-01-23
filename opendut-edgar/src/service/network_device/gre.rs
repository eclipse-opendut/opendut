use std::net::Ipv4Addr;
use std::rc::Rc;
use anyhow::Context;
use futures::executor::block_on;
use opendut_types::util::net::NetworkInterfaceName;
use crate::service::network_device::manager::NetworkDeviceManager;

const GRE_INTERFACE_NAME_PREFIX: &str = "gre-opendut";


pub async fn remove_existing_interfaces(network_device_manager: &Rc<NetworkDeviceManager>) -> anyhow::Result<()> {
    let interfaces_to_remove = block_on(network_device_manager.list_interfaces())?
        .into_iter()
        .filter(|interface| interface.name.name().starts_with(GRE_INTERFACE_NAME_PREFIX));

    for interface in interfaces_to_remove {
        network_device_manager.delete_interface(&interface).await?;
    }

    Ok(())
}


pub async fn create_interface(
    local_ip: Ipv4Addr,
    remote_ip: Ipv4Addr,
    interface_index: usize,
    bridge_name: &NetworkInterfaceName,
    network_device_manager: &Rc<NetworkDeviceManager>,
) -> anyhow::Result<()> {
    let interface_prefix = GRE_INTERFACE_NAME_PREFIX;
    let interface_name = NetworkInterfaceName::try_from(format!("{}{}", interface_prefix, interface_index))
        .context("Error while constructing GRE interface name")?;

    let gre_interface = network_device_manager.create_gretap_v4_interface(&interface_name, &local_ip, &remote_ip).await?;
    log::trace!("Created GRE interface '{gre_interface}'.");
    network_device_manager.set_interface_up(&gre_interface).await?;
    log::trace!("Set GRE interface '{interface_name}' to 'up'.");

    let bridge = network_device_manager.try_find_interface(bridge_name).await?;
    network_device_manager.join_interface_to_bridge(&gre_interface, &bridge).await?;

    Ok(())
}
