use async_trait::async_trait;
use opendut_types::peer::configuration::{parameter, Parameter, ParameterTarget};
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub struct ManageJoinedInterface {
    pub parameter: Parameter<parameter::InterfaceJoinConfig>,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}


#[async_trait]
impl Task for ManageJoinedInterface {
    fn description(&self) -> String {
        format!("Manage interface '{}' join configuration.", self.parameter.value)
    }

    async fn check_fulfilled(&self) -> anyhow::Result<TaskFulfilled> {
        // TODO: check pre-requisite that bridge interface needs to be present
        let joined = self.network_interface_manager
            .find_interfaces_joined_to_bridge(&self.parameter.value.bridge).await?
            .into_iter()
            .find(|interface| interface.name == self.parameter.value.name);
        
        match (joined, self.parameter.target) {
            (Some(_), ParameterTarget::Absent) | (None, ParameterTarget::Present) => {
                Ok(TaskFulfilled::No)
            }
            (Some(_), ParameterTarget::Present) | (None, ParameterTarget::Absent) => {
                Ok(TaskFulfilled::Yes)
            }
        }
    }

    async fn execute(&self) -> anyhow::Result<Success> {
        let joined = self.network_interface_manager
            .find_interfaces_joined_to_bridge(&self.parameter.value.bridge).await?
            .into_iter()
            .find(|interface| interface.name == self.parameter.value.name);

        match (joined, self.parameter.target) {
            (Some(name), ParameterTarget::Absent) => {
                self.network_interface_manager.remove_interface_from_bridge(&name).await?;
            }
            (None, ParameterTarget::Present) => {
                let interface = self.network_interface_manager.find_interface(&self.parameter.value.name).await?;
                let bridge = self.network_interface_manager.find_interface(&self.parameter.value.bridge).await?;
                if let (Some(interface), Some(bridge)) = (interface, bridge) {
                    self.network_interface_manager.join_interface_to_bridge(&interface, &bridge).await?;
                } 
            }
            (Some(_), ParameterTarget::Present) | (None, ParameterTarget::Absent) => {
                // nothing to do
            }
        }
        Ok(Success::default())

    }
}
