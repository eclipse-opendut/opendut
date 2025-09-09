use anyhow::Result;
use async_trait::async_trait;
use opendut_model::util::net::NetworkInterfaceName;

use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use crate::common::task::{Success, Task, TaskStateFulfilled};

pub struct CreateBridge {
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub bridge_name: NetworkInterfaceName,
}

#[async_trait]
impl Task for CreateBridge {
    fn description(&self) -> String {
        format!("Create Bridge \"{}\"", self.bridge_name)
    }
    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        let bridge_exists = self.network_interface_manager.find_interface(&self.bridge_name).await?
            .is_some();

        if bridge_exists {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }
    async fn make_present(&self) -> Result<Success> {
        let bridge = self.network_interface_manager.create_empty_bridge(&self.bridge_name).await?;
        self.network_interface_manager.set_opendut_alternative_name(&bridge).await?;

        Ok(Success::default())
    }
}
