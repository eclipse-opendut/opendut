use std::process::Command;

use anyhow::{Context, Result};

use crate::setup::constants;
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

/// The EDGAR Service needs to modify network interfaces.
/// This tasks requests the Linux Capability "CAP_NET_ADMIN", which allows doing so without root permissions.
pub struct RequestLinuxNetworkCapability;
impl Task for RequestLinuxNetworkCapability {
    fn description(&self) -> String {
        String::from("Linux Network Capability")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let getcap = which::which("getcap")
            .context(String::from("No command `getcap` found. Ensure your system provides this command."))?;

        let output = Command::new(getcap)
            .arg(constants::executable_install_path()?)
            .evaluate_requiring_success()
            .context(format!("Error while determining Linux Capabilities of executable at: {}", constants::executable_install_path()?.display()))?;

        if output.stdout.is_empty() {
            Ok(TaskFulfilled::No)
        } else {
            Ok(TaskFulfilled::Yes)
        }
    }
    fn execute(&self) -> Result<Success> {
        let setcap = which::which("setcap")
            .context(String::from("No command `setcap` found. Ensure your system provides this command."))?;

        let _ = Command::new(setcap)
            .arg("CAP_NET_ADMIN+pe") //"permitted" and "effective", see `man capabilities 7` -> "File capabilities"
            .arg(constants::executable_install_path()?)
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
