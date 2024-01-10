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
        let bridge = block_on(self.network_device_manager.create_empty_bridge(&self.bridge_name))?;
        block_on(self.network_device_manager.set_interface_up(&bridge))?;
        Ok(Success::default())
    }
}
