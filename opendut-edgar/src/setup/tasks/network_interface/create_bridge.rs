use std::sync::Arc;
use anyhow::Result;
use futures::executor::block_on;

use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct CreateBridge {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub bridge_name: NetworkInterfaceName,
}
impl Task for CreateBridge {
    fn description(&self) -> String {
        format!("Create Bridge \"{}\"", self.bridge_name)
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let bridge_exists = block_on(self.network_interface_manager.find_interface(&self.bridge_name))?
            .is_some();

        if bridge_exists {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        block_on(crate::service::network_interface::bridge::create(&self.bridge_name, Arc::clone(&self.network_interface_manager)))?;

        Ok(Success::default())
    }
}
