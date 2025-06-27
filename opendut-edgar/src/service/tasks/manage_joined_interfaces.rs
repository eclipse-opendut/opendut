use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::interface::Interface;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use async_trait::async_trait;
use opendut_types::peer::configuration::parameter;

pub struct ManageJoinedInterface {
    pub parameter: parameter::InterfaceJoinConfig,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

impl ManageJoinedInterface {
    async fn find_joined_interface(&self) -> anyhow::Result<Option<Interface>> {
        let joined = self.network_interface_manager
            .find_interfaces_joined_to_bridge(&self.parameter.bridge).await?
            .into_iter()
            .find(|interface| interface.name == self.parameter.name);
        
        Ok(joined)
    }
}

#[async_trait]
impl Task for ManageJoinedInterface {
    fn description(&self) -> String {
        format!("Manage interface '{}' join configuration.", self.parameter)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        if self.find_joined_interface().await?.is_some() {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        if self.find_joined_interface().await?.is_none() {
            let interface = self.network_interface_manager.find_interface(&self.parameter.name).await?;
            let bridge = self.network_interface_manager.find_interface(&self.parameter.bridge).await?;
            if let (Some(interface), Some(bridge)) = (interface, bridge) {
                self.network_interface_manager.join_interface_to_bridge(&interface, &bridge).await?;
            } else {
                return Err(anyhow::Error::msg(format!(
                    "Cannot join interface '{}' to bridge '{}': one of them does not exist.",
                    self.parameter.name, self.parameter.bridge
                )));
            }
        }
        Ok(Success::default())
    }
}

#[async_trait]
impl TaskAbsent for ManageJoinedInterface {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        if self.find_joined_interface().await?.is_some() {
            Ok(TaskStateFulfilled::No)
        } else {
            Ok(TaskStateFulfilled::Yes)
        }
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        if let Some(interface) = self.find_joined_interface().await? {
            self.network_interface_manager.remove_interface_from_bridge(&interface).await?;
        }
        Ok(Success::default())
    }
}