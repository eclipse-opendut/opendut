use std::process::Command;

use anyhow::Result;
use async_trait::async_trait;
use crate::setup::constants::SYSTEMD_SERVICE_FILE_NAME;
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

pub struct RestartService;

#[async_trait]
impl Task for RestartService {
    fn description(&self) -> String {
        String::from("(Re-)Start Service")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    async fn execute(&self) -> Result<Success> {
        Command::new("systemctl")
            .arg("stop")
            .arg(SYSTEMD_SERVICE_FILE_NAME)
            .evaluate_requiring_success()?; //systemctl always return status code 0, even if already stopped

        Command::new("systemctl")
            .args(["enable", "--now"]) //enable and start
            .arg(SYSTEMD_SERVICE_FILE_NAME)
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
