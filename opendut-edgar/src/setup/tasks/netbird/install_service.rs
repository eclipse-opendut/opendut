use std::process::Command;

use anyhow::Result;
use async_trait::async_trait;
use crate::setup::constants::netbird;
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

pub struct InstallService;

#[async_trait]
impl Task for InstallService {
    fn description(&self) -> String {
        String::from("NetBird - Install Service")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let output = Command::new("systemctl")
            .arg("cat")
            .arg("netbird")
            .output()?;

        if output.status.success() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    async fn execute(&self) -> Result<Success> {
        let netbird = netbird::unpacked_executable()?;

        let _ = Command::new(netbird)
            .arg("service")
            .arg("install")
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
