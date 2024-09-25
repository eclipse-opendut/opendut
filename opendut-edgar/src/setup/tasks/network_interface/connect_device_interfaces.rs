use std::collections::HashSet;

use anyhow::Result;
use async_trait::async_trait;
use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::common::task::{Success, Task, TaskFulfilled};

pub struct ConnectDeviceInterfaces {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub bridge_name: NetworkInterfaceName,
    pub device_interfaces: HashSet<NetworkInterfaceName>,
}

#[async_trait]
impl Task for ConnectDeviceInterfaces {
    fn description(&self) -> String {
        String::from("Connect Interfaces of Configured Test Devices")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    async fn execute(&self) -> Result<Success> {
        let bridge = self.network_interface_manager.try_find_interface(&self.bridge_name).await?;

        for interface in &self.device_interfaces {
            let interface = self.network_interface_manager.try_find_interface(interface).await?;
            self.network_interface_manager.join_interface_to_bridge(&interface, &bridge).await?;
        }
        Ok(Success::default())
    }
}
