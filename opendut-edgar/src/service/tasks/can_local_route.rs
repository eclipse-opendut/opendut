use std::fmt::Display;
use async_trait::async_trait;
use regex::Regex;
use tokio::process::Command;
use opendut_model::peer::configuration::parameter;
use opendut_model::util::net::NetworkInterfaceName;
use crate::common::task::{Success, Task, TaskAbsent, TaskStateFulfilled};
use crate::service::network_interface::manager::NetworkInterfaceManagerRef;

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
        format!("Create local CAN route from '{}' to bridge '{}'.", self.parameter.can_device_name, self.parameter.bridge_name)
    }

    async fn check_present(&self) -> anyhow::Result<TaskStateFulfilled> {
        let can_device = self.network_interface_manager.find_interface(&self.parameter.can_device_name).await?;
        let bridge = self.network_interface_manager.find_interface(&self.parameter.bridge_name).await?;

        match (can_device, bridge) {
            (Some(_), Some(_)) => {
                let can_route_present = self.check_can_route_exists(
                    &self.parameter.can_device_name,
                    &self.parameter.bridge_name,
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
        let can_device = self.network_interface_manager.find_interface(&self.parameter.can_device_name).await?;
        let bridge = self.network_interface_manager.find_interface(&self.parameter.bridge_name).await?;

        match (can_device, bridge) {
            (Some(can_device), Some(bridge)) => {
                self.create_can_route(
                    &can_device.name,
                    &bridge.name,
                    self.can_fd,
                    CAN_MAX_HOPS,
                    CanRouteOperation::Create,
                ).await?;

                Ok(Success::default())
            },
            _ => Err(anyhow::Error::msg("Cannot create CAN local route because either CAN device or bridge does not exist.")),
        }
    }
}

#[async_trait]
impl TaskAbsent for CanLocalRoute {
    async fn check_absent(&self) -> anyhow::Result<TaskStateFulfilled> {
        let can_route_present = self.check_can_route_exists(
            &self.parameter.can_device_name,
            &self.parameter.bridge_name,
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
            &self.parameter.can_device_name,
            &self.parameter.bridge_name,
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

        let output = cmd.output().await
            .map_err(|cause| Error::CommandLineProgramExecution { command: "cangw".to_string(), cause })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::CanRouteCreation {
                src: src.clone(),
                dst: dst.clone(),
                operation,
                cause: format!("{:?}", String::from_utf8_lossy(&output.stderr).trim())
            })
        }

    }
}