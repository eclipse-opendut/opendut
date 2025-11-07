use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::can::can_manager::CanManagerRef;
use opendut_model::peer::configuration::parameter;

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
        if self.is_running().await {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        if !self.is_running().await {
            let can_manager = self.can_manager.lock().await;
            can_manager.spawn_process(&self.parameter).await?;
            Ok(Success::message(format!("CAN connection to {} established", self.parameter.remote_peer_id)))
        } else {
            Ok(Success::message(format!("CAN connection to {} was already running", self.parameter.remote_peer_id)))
        }
    }
}

#[async_trait::async_trait]
impl TaskAbsent for CanConnection {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        if self.is_running().await {
            Ok(TaskStateFulfilled::No)
        } else {
            Ok(TaskStateFulfilled::Yes)
        }
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        if self.is_running().await {
            let can_manager = self.can_manager.lock().await;
            can_manager.terminate_process(&self.parameter).await?;
            Ok(Success::message(format!("CAN connection to {} terminated", self.parameter.remote_peer_id)))
        } else {
            Ok(Success::message(format!("CAN connection to {} was not running", self.parameter.remote_peer_id)))
        }
    }
}

impl CanConnection {
    async fn is_running(&self) -> bool {
        let can_manager = self.can_manager.lock().await;
        can_manager.process_is_running(&self.parameter).await
    }
}