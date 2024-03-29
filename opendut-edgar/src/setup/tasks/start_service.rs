use std::process::Command;

use anyhow::Result;

use crate::setup::constants::SYSTEMD_SERVICE_FILE_NAME;
use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

pub struct StartService;
impl Task for StartService {
    fn description(&self) -> String {
        String::from("Start Service")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg(SYSTEMD_SERVICE_FILE_NAME)
            .output()?;

        if output.status.success() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        let _ = Command::new("systemctl")
            .args(["enable", "--now"]) //enable and start
            .arg(SYSTEMD_SERVICE_FILE_NAME)
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
