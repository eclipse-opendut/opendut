use std::collections::HashSet;

use anyhow::Result;
use config::Config;
use futures::executor::block_on;

use opendut_types::topology::Topology;
use opendut_types::util::net::NetworkInterfaceName;

use crate::common::settings;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct ConnectDeviceInterfaces {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub bridge_name: NetworkInterfaceName,
}
impl Task for ConnectDeviceInterfaces {
    fn description(&self) -> String {
        String::from("Connect Interfaces of Configured Test Devices")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {
        let settings = settings::load_with_overrides(Config::default()).expect("Failed to load configuration.");

        let topology = settings.config
            .get::<Topology>("topology")
            .expect("Unable to load topology from configuration");

        let bridge = block_on(self.network_interface_manager.try_find_interface(&self.bridge_name))?;

        let interfaces = topology.devices
            .into_iter()
            .map(|device| device.interface)
            .collect::<HashSet<_>>();

        for interface in interfaces {
            let interface = block_on(self.network_interface_manager.try_find_interface(&interface))?;
            block_on(self.network_interface_manager.join_interface_to_bridge(&interface, &bridge))?;
        }
        Ok(Success::default())
    }
}
