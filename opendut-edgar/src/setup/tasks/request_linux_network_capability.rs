use std::process::Command;

use anyhow::{Context, Result};
use async_trait::async_trait;
use crate::setup::constants;
use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

/// The EDGAR Service needs to modify network interfaces.
/// This tasks requests the Linux Capability "CAP_NET_ADMIN", which allows doing so without root permissions.
pub struct RequestLinuxNetworkCapability;

#[async_trait]
impl Task for RequestLinuxNetworkCapability {
    fn description(&self) -> String {
        String::from("Linux Network Capability")
    }
    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        let getcap = which::which("getcap")
            .context(String::from("No command `getcap` found. Ensure your system provides this command."))?;

        let output = Command::new(getcap)
            .arg(constants::executable_install_path()?)
            .evaluate_requiring_success()
            .context(format!("Error while determining Linux Capabilities of executable at: {}", constants::executable_install_path()?.display()))?;

        if output.stdout.is_empty() {
            Ok(TaskStateFulfilled::No)
        } else {
            Ok(TaskStateFulfilled::Yes)
        }
    }
    async fn make_present(&self) -> Result<Success> {
        let setcap = which::which("setcap")
            .context(String::from("No command `setcap` found. Ensure your system provides this command."))?;

        let _ = Command::new(setcap)
            .arg("CAP_NET_ADMIN+pe") //"permitted" and "effective", see `man capabilities 7` -> "File capabilities"
            .arg(constants::executable_install_path()?)
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
