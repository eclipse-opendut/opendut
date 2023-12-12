use std::process::Command;

use anyhow::Result;

use crate::setup::constants;
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::evaluate_requiring_success;

pub struct RequestCapabilityForExecutable;
impl Task for RequestCapabilityForExecutable {
    fn description(&self) -> String {
        String::from("Linux Network Capability - Request for Executable")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let mut command = Command::new("getcap");
        let command = command
            .arg(constants::executable_install_path()?);

        let output = evaluate_requiring_success(command)?;

        if output.stdout.is_empty() {
            Ok(TaskFulfilled::No)
        } else {
            Ok(TaskFulfilled::Yes)
        }
    }
    fn execute(&self) -> Result<Success> {
        let mut command = Command::new("setcap");
        let command = command
            .arg("CAP_NET_ADMIN=ei") //"effective" and "inheritable"
            .arg(constants::executable_install_path()?);

        let _ = evaluate_requiring_success(command)?;
        Ok(Success::default())
    }
}
