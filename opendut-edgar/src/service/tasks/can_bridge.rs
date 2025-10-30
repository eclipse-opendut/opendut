use async_trait::async_trait;
use opendut_model::peer::configuration::parameter;
use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub struct CanBridge {
    pub parameter: parameter::CanBridge,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

#[async_trait]
impl Task for CanBridge {
    fn description(&self) -> String {
        format!("Create CAN bridge '{}'", self.parameter.name)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        todo!()
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        todo!()
    }
}