use std::collections::HashSet;

use anyhow::Result;
use futures::executor::block_on;

use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct ConnectDeviceInterfaces {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub bridge_name: NetworkInterfaceName,
    pub device_interfaces: HashSet<NetworkInterfaceName>,
}
impl Task for ConnectDeviceInterfaces {
    fn description(&self) -> String {
        String::from("Connect Interfaces of Configured Test Devices")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {
        let bridge = block_on(self.network_interface_manager.try_find_interface(&self.bridge_name))?;

        for interface in &self.device_interfaces {
            let interface = block_on(self.network_interface_manager.try_find_interface(interface))?;
            block_on(self.network_interface_manager.join_interface_to_bridge(&interface, &bridge))?;
        }
        Ok(Success::default())
    }
}
