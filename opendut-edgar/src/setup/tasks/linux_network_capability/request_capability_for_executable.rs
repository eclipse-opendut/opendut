use std::process::Command;

use anyhow::Result;

use crate::setup::constants;
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

pub struct RequestCapabilityForExecutable;
impl Task for RequestCapabilityForExecutable {
    fn description(&self) -> String {
        String::from("Linux Network Capability - Request for Executable")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let output = Command::new("getcap")
            .arg(constants::executable_install_path()?)
            .evaluate_requiring_success()?;

        if output.stdout.is_empty() {
            Ok(TaskFulfilled::No)
        } else {
            Ok(TaskFulfilled::Yes)
        }
    }
    fn execute(&self) -> Result<Success> {
        let _ = Command::new("setcap")
            .arg("CAP_NET_ADMIN=ei") //"effective" and "inheritable"
            .arg(constants::executable_install_path()?)
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
