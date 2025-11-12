use anyhow::anyhow;
use tracing::error;
use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::can::CanInterfaceConfiguration;
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use opendut_model::util::net::NetworkInterfaceName;
use crate::service::network_interface::manager::interface::{Interface, NetlinkInterfaceKind};

pub struct CanDeviceConfiguration {
    pub interface_name: NetworkInterfaceName,
    pub can_config: CanInterfaceConfiguration,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

#[async_trait::async_trait]
impl Task for CanDeviceConfiguration {
    fn description(&self) -> String {
        format!("CAN device <{}> configuration: {:?}", self.interface_name, self.can_config)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let interface = self.find_interface().await?;
        if interface.kind == NetlinkInterfaceKind::Vcan { //TODO treat as error, if we make it impossible for this to be CAN interface upfront
            return Ok(TaskStateFulfilled::Yes);  // Virtual CAN interfaces are assumed to always match the desired configuration.
        }
        // This assumes the device exists otherwise an error is returned.
        let detected_can_config = self.network_interface_manager
            .detect_can_device_configuration(self.interface_name.clone())
            .await?;

        if detected_can_config == self.can_config {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        let interface = self.find_interface().await?;
        self.network_interface_manager.set_interface_down(&interface).await?;

        if let Err(cause) = self.network_interface_manager.update_can_interface(&self.interface_name, &self.can_config).await { //TODO make this a hard error
            error!("Error updating CAN interface - A possible reason might be, that a virtual CAN interface was used: {cause}");
        };
        self.network_interface_manager.set_interface_up(&interface).await?;

        Ok(Success::default())
    }
}

impl CanDeviceConfiguration {
    async fn find_interface(&self) -> anyhow::Result<Interface> {
        let interface = self.network_interface_manager.find_interface(&self.interface_name)
            .await?
            .ok_or_else(|| anyhow!("Cannot find network interface with name {}", self.interface_name))?;
        Ok(interface)
    }
}

#[async_trait::async_trait]
impl TaskAbsent for CanDeviceConfiguration {

    /// No action needed to check CAN device configuration absence.
    /// It is assumed that another task overwrites the configuration or removes the interface entirely.
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        Ok(TaskStateFulfilled::Unchecked)
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        // No action needed to remove CAN device configuration.
        Ok(Success::default())
    }
}
