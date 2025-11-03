use async_trait::async_trait;
use opendut_model::util::net::NetworkInterfaceName;
use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

pub struct CanCreateVirtualDevice {
    pub name: NetworkInterfaceName,
    pub network_interface_manager: NetworkInterfaceManagerRef,
}

#[async_trait]
impl Task for CanCreateVirtualDevice {
    fn description(&self) -> String {
        format!("Create vCAN device '{}'", self.name)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let interface = self.network_interface_manager.find_interface(&self.name).await?;
        match interface {
            Some(vcan) => {
                let interface_is_up = vcan.link_flags.contains(rtnetlink::packet_route::link::LinkFlags::Up);
                if vcan.kind == crate::service::network_interface::manager::interface::NetlinkInterfaceKind::VCan && interface_is_up {
                    Ok(TaskStateFulfilled::Yes)
                } else {
                    Ok(TaskStateFulfilled::No)
                }
            }
            None => Ok(TaskStateFulfilled::No),
        }

    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        let interface = self.network_interface_manager.find_interface(&self.name).await?;
        match interface {
            None => {
                let vcan = self.network_interface_manager.create_vcan_interface(&self.name).await?;
                self.network_interface_manager.set_opendut_alternative_name(&vcan).await?;
                self.network_interface_manager.set_interface_up(&vcan).await?;

                Ok(Success::default())
            }
            Some(vcan) => {
                if vcan.kind == crate::service::network_interface::manager::interface::NetlinkInterfaceKind::VCan {
                    self.network_interface_manager.set_interface_up(&vcan).await?;
                    Ok(Success::default())
                } else {
                    Err(anyhow::Error::msg(format!("Another interface with that name exists but it has an unexpected interface kind: <{:?}>!", vcan.kind)))
                }

            }
        }

    }
}

#[async_trait]
impl TaskAbsent for CanCreateVirtualDevice {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        let interface = self.network_interface_manager.find_interface(&self.name).await?;
        match interface {
            None => Ok(TaskStateFulfilled::Yes),
            Some(_) => Ok(TaskStateFulfilled::No),
        }
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        if let Some(interface) = self.network_interface_manager.find_interface(&self.name).await? {
            self.network_interface_manager.delete_interface(&interface).await?;
        }
        Ok(Success::default())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    mod present {
        use crate::common::task::Task;
        use crate::service::tasks::can_local_route::tests::FixtureVirtualCan;
        use crate::service::tasks::can_virtual_device::CanCreateVirtualDevice;

        #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
        #[test_log::test(tokio::test)]
        async fn test_can_virtual_device_present() -> anyhow::Result<()> {
            // Arrange
            let fixture = FixtureVirtualCan::create().await.expect("Could not create FixtureVirtualCan");
            fixture.verify_required_linux_kernel_modules_are_loaded()?;
            fixture.create_vcan_interfaces().await?;

            let task = CanCreateVirtualDevice {
                name: fixture.vcan1_name.clone(),
                network_interface_manager: fixture.network_interface_manager.clone(),
            };

            // Act
            let result = task.check_present().await?;

            // Assert
            assert!(matches!(result, crate::common::task::TaskStateFulfilled::Yes), "Expected vCAN device to be present after creation.");

            Ok(())
        }

        #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
        #[test_log::test(tokio::test)]
        async fn test_can_virtual_device_not_present() -> anyhow::Result<()> {
            // Arrange
            let fixture = FixtureVirtualCan::create().await.expect("Could not create FixtureVirtualCan");
            fixture.verify_required_linux_kernel_modules_are_loaded()?;

            let task = CanCreateVirtualDevice {
                name: fixture.vcan1_name.clone(),
                network_interface_manager: fixture.network_interface_manager.clone(),
            };

            // Act
            let result = task.check_present().await?;

            // Assert
            assert!(matches!(result, crate::common::task::TaskStateFulfilled::No), "Checking non-existing vCAN device should evaluate to 'no'.");

            Ok(())
        }

    }

    mod absent {
        use crate::common::task::TaskAbsent;
        use crate::service::tasks::can_local_route::tests::FixtureVirtualCan;
        use crate::service::tasks::can_virtual_device::CanCreateVirtualDevice;

        #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
        #[test_log::test(tokio::test)]
        async fn test_vcan_existing_virtual_device_is_not_absent() -> anyhow::Result<()> {
            // Arrange
            let fixture = FixtureVirtualCan::create().await.expect("Could not create FixtureVirtualCan");
            fixture.verify_required_linux_kernel_modules_are_loaded()?;
            fixture.create_vcan_interfaces().await?;

            let task = CanCreateVirtualDevice {
                name: fixture.vcan1_name.clone(),
                network_interface_manager: fixture.network_interface_manager.clone(),
            };

            // Act
            let result = task.check_absent().await?;

            // Assert
            assert!(matches!(result, crate::common::task::TaskStateFulfilled::No), "Checking present device for absence should evaluate to 'no'.");

            Ok(())
        }

        #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
        #[test_log::test(tokio::test)]
        async fn test_vcan_not_existing_virtual_device_is_absent() -> anyhow::Result<()> {
            // Arrange
            let fixture = FixtureVirtualCan::create().await.expect("Could not create FixtureVirtualCan");
            fixture.verify_required_linux_kernel_modules_are_loaded()?;

            let task = CanCreateVirtualDevice {
                name: fixture.vcan1_name.clone(),
                network_interface_manager: fixture.network_interface_manager.clone(),
            };

            // Act
            let result = task.check_absent().await?;

            // Assert
            assert!(matches!(result, crate::common::task::TaskStateFulfilled::Yes), "Checking absent device for absence should evaluate to 'yes'.");

            Ok(())
        }
    }


}