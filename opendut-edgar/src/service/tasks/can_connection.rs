use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::can::can_manager::CanManagerRef;
use opendut_model::peer::configuration::{parameter, ParameterValue};

pub struct CanConnection {
    pub parameter: parameter::CanConnection,
    pub can_manager: CanManagerRef,
}

#[async_trait::async_trait]
impl Task for CanConnection {
    fn description(&self) -> String {
        format!("CAN connection to peer <{}> via interface <{}>", self.parameter.remote_peer_id, self.parameter.can_interface_name)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let mut can_manager = self.can_manager.lock().await;
        let is_running = can_manager.process_is_running(&self.parameter).await?;

        if is_running {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        let id = self.parameter.parameter_identifier();


        Ok(Success::message(format!("CAN connection to {} established", self.parameter.remote_peer_id)))
    }
}

#[async_trait::async_trait]
impl TaskAbsent for CanConnection {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        todo!()
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        todo!()
    }
}
