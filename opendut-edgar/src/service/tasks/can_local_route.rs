use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;
use anyhow::anyhow;
use async_trait::async_trait;
use opendut_model::peer::configuration::parameter;
use opendut_model::util::net::NetworkInterfaceName;
use regex::Regex;
use std::fmt::Display;
use tokio::process::Command;
use tracing::trace;

pub struct CanLocalRoute {
    pub parameter: parameter::CanLocalRoute,
    pub network_interface_manager: NetworkInterfaceManagerRef,
    pub can_fd: bool,
}

/// Maximum number of hops for local CAN message.
const CAN_MAX_HOPS: u8 = 2;

#[derive(Debug)]
pub enum CanRouteOperation {
    Create,
    Delete,
}
impl Display for CanRouteOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CanRouteOperation::Create => write!(f, "create"),
            CanRouteOperation::Delete => write!(f, "delete"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failure while invoking command line program '{command}': {cause}")]
    CommandLineProgramExecution { command: String, cause: std::io::Error },
    #[error("Failed to {operation} CAN route '{src}' -> '{dst}': {cause}")]
    CanRouteCreation { src: NetworkInterfaceName, dst: NetworkInterfaceName, operation: CanRouteOperation, cause: String },
}


#[async_trait]
impl Task for CanLocalRoute {
    fn description(&self) -> String {
        format!("Create local CAN route from '{}' to bridge '{}'.", self.parameter.can_destination_device_name, self.parameter.can_source_device_name)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let source = self.network_interface_manager.find_interface(&self.parameter.can_source_device_name).await?;
        let destination = self.network_interface_manager.find_interface(&self.parameter.can_destination_device_name).await?;

        match (source, destination) {
            (Some(_), Some(_)) => {
                let can_route_present = self.check_can_route_exists(
                    &self.parameter.can_source_device_name,
                    &self.parameter.can_destination_device_name,
                    self.can_fd,
                    CAN_MAX_HOPS,
                ).await?;

                if can_route_present {
                    Ok(TaskStateFulfilled::Yes)
                } else {
                    Ok(TaskStateFulfilled::No)
                }
            },
            _ => Ok(TaskStateFulfilled::No),
        }
    }

    async fn make_present(&self) -> anyhow::Result<Success> {
        let source = self.network_interface_manager.find_interface(&self.parameter.can_source_device_name).await?;
        let destination = self.network_interface_manager.find_interface(&self.parameter.can_destination_device_name).await?;

        match (source, destination) {
            (Some(source), Some(destination)) => {
                self.create_can_route(
                    &source.name,
                    &destination.name,
                    self.can_fd,
                    CAN_MAX_HOPS,
                    CanRouteOperation::Create,
                ).await?;

                Ok(Success::default())
            },
            _ => Err(anyhow::Error::msg("Cannot create CAN local route because either source or destination does not exist.")),
        }
    }
}

#[async_trait]
impl TaskAbsent for CanLocalRoute {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        let can_route_present = self.check_can_route_exists(
            &self.parameter.can_source_device_name,
            &self.parameter.can_destination_device_name,
            self.can_fd,
            CAN_MAX_HOPS,
        ).await?;

        if can_route_present {
            Ok(TaskStateFulfilled::No)
        } else {
            Ok(TaskStateFulfilled::Yes)
        }
    }

    async fn make_absent(&self) -> anyhow::Result<Success> {
        self.create_can_route(
            &self.parameter.can_source_device_name,
            &self.parameter.can_destination_device_name,
            self.can_fd,
            CAN_MAX_HOPS,
            CanRouteOperation::Delete,
        ).await?;

        Ok(Success::default())
    }
}

impl CanLocalRoute {
    async fn check_can_route_exists(&self, src: &NetworkInterfaceName, dst: &NetworkInterfaceName, can_fd: bool, max_hops: u8) -> anyhow::Result<bool> {
        let output = Command::new("cangw")
            .arg("-L")
            .output()
            .await
            .map_err(|cause| Error::CommandLineProgramExecution { command: "cangw".to_string(), cause })?;

        // cangw -L returns non-zero exit code despite succeeding, so we don't check it here

        let output_str = String::from_utf8_lossy(&output.stdout);
        println!("cangw -L output:\n{}", output_str);

        let regex = Regex::new(r"(?m)^cangw -A -s ([^\n ]+) -d ([^\n ]+) ((?:-X )?)-e -l ([0-9[^\n ]]+) #.*$").unwrap();

        let captures = regex.captures_iter(&output_str).map(|captures| captures.extract().1);

        for [exist_src, exist_dst, can_fd_flag, exist_max_hops] in captures {
            let exist_can_fd = can_fd_flag.trim() == "-X";

            if exist_src == src.name()
                && exist_dst == dst.name()
                && exist_can_fd == can_fd
                && exist_max_hops == max_hops.to_string() {
                return Ok(true)
            }
        }

        Ok(false)
    }

    async fn create_can_route(&self, src: &NetworkInterfaceName, dst: &NetworkInterfaceName, can_fd: bool, max_hops: u8, operation: CanRouteOperation) -> anyhow::Result<()> {
        let operation_arg = match operation {
            CanRouteOperation::Create => "-A",
            CanRouteOperation::Delete => "-D",
        };

        let mut cmd = Command::new("cangw");
        cmd.arg(operation_arg)
            .arg("-s")
            .arg(src.name())
            .arg("-d")
            .arg(dst.name())
            .arg("-e")
            .arg("-l")
            .arg(max_hops.to_string());

        if can_fd {
            cmd.arg("-X");
        }

        trace!("{operation:?} CAN route, executing command: {:?}", cmd);
        let output = cmd.output().await
            .map_err(|cause| anyhow!(Error::CommandLineProgramExecution { command: "cangw".to_string(), cause }))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(anyhow!(Error::CanRouteCreation {
                src: src.clone(),
                dst: dst.clone(),
                operation,
                cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim())
            }))
        }

    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::service::network_interface::manager::{NetworkInterfaceManager, NetworkInterfaceManagerRef};
    use opendut_model::peer::configuration::parameter;
    use opendut_model::util::net::NetworkInterfaceName;
    use std::sync::Arc;
    use anyhow::anyhow;
    use crate::common::task::{Task, TaskAbsent, TaskStateFulfilled};
    use crate::service::tasks::testing::NetworkInterfaceNameExt;

    pub struct Fixture {
        network_interface_manager: NetworkInterfaceManagerRef,
        parameter: parameter::CanLocalRoute,
        vcan1_name: NetworkInterfaceName,
        vcan2_name: NetworkInterfaceName,
    }
    impl Fixture {
        pub async fn create() -> anyhow::Result<Self> {
            let (connection, handle, _) = rtnetlink::new_connection().expect("Could not get rtnetlink handle.");
            tokio::spawn(connection);
            let manager = NetworkInterfaceManager { handle };
            let network_interface_manager = Arc::new(manager);
            let vcan1_name = NetworkInterfaceName::with_random_suffix("vcan1");
            let vcan2_name = NetworkInterfaceName::with_random_suffix("vcan2");

            let parameter = parameter::CanLocalRoute {
                can_source_device_name: vcan1_name.clone(),
                can_destination_device_name: vcan2_name.clone(),
            };

            // Verify that required CAN kernel modules are loaded
            for kernel_module in opendut_edgar_kernel_modules::required_can_kernel_modules() {
                if ! kernel_module.is_loaded(&opendut_edgar_kernel_modules::default_module_file(), &opendut_edgar_kernel_modules::default_builtin_module_dir())? {
                    return Err(anyhow!("Required CAN kernel module '{}' is not loaded. Cannot run CAN local route tests.", kernel_module.name()))
                }
            }

            Ok(Self {
                network_interface_manager,
                parameter,
                vcan1_name,
                vcan2_name,
            })
        }
        async fn create_vcan_interfaces(&self) -> anyhow::Result<()> {
            let vcan1_interface = self.network_interface_manager.create_vcan_interface(&self.vcan1_name).await?;
            self.network_interface_manager.set_interface_up(&vcan1_interface).await?;
            let vcan2_interface = self.network_interface_manager.create_vcan_interface(&self.vcan2_name).await?;
            self.network_interface_manager.set_interface_up(&vcan2_interface).await?;

            let found_vcan1 = self.network_interface_manager.find_interface(&self.vcan1_name).await?;
            let found_vcan2 = self.network_interface_manager.find_interface(&self.vcan2_name).await?;
            assert!(found_vcan1.is_some());
            assert!(found_vcan2.is_some());
            Ok(())
        }
    }
    #[test_log::test(tokio::test)]
    async fn test_can_local_route_description() {
        let fixture = Fixture::create().await.expect("Could not create Fixture");
        let task = super::CanLocalRoute {
            parameter: fixture.parameter.clone(),
            network_interface_manager: fixture.network_interface_manager.clone(),
            can_fd: false,
        };
        let description = task.description();
        assert_eq!(
            description,
            format!(
                "Create local CAN route from '{}' to bridge '{}'.",
                fixture.parameter.can_destination_device_name,
                fixture.parameter.can_source_device_name
            )
        );
    }

    #[test_with::env(RUN_EDGAR_NETLINK_INTEGRATION_TESTS)]
    #[test_log::test(tokio::test)]
    async fn test_can_local_route_lifecycle() -> anyhow::Result<()> {
        let fixture = Fixture::create().await.expect("Could not create Fixture");
        fixture.create_vcan_interfaces().await?;

        let task = super::CanLocalRoute {
            parameter: fixture.parameter.clone(),
            network_interface_manager: fixture.network_interface_manager.clone(),
            can_fd: false,
        };

        // Ensure absent
        if let Ok(present) = task.check_present().await
            && present == TaskStateFulfilled::Yes {
            task.make_absent().await?;
        }

        // Create CAN local route
        task.make_present().await?;

        // Verify present
        let present = task.check_present().await?;
        assert_eq!(present, TaskStateFulfilled::Yes);

        // Remove CAN local route
        task.make_absent().await?;

        // Verify absent
        let absent = task.check_absent().await?;
        assert_eq!(absent, TaskStateFulfilled::Yes);

        Ok(())
    }
}