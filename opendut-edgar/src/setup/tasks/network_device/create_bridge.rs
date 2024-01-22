use std::rc::Rc;

use anyhow::Result;
use futures::executor::block_on;

use opendut_types::util::net::NetworkInterfaceName;

use crate::service::network_device::manager::NetworkDeviceManager;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct CreateBridge {
    pub network_device_manager: Rc<NetworkDeviceManager>,
    pub bridge_name: NetworkInterfaceName,
}
impl Task for CreateBridge {
    fn description(&self) -> String {
        format!("Create Bridge \"{}\"", self.bridge_name)
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let bridge_exists = block_on(self.network_device_manager.find_interface(&self.bridge_name))?
            .is_some();

        if bridge_exists {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        block_on(crate::service::network_device::bridge::create(&self.bridge_name, &self.network_device_manager))?;

        Ok(Success::default())
    }
}
