use std::process::Command;

use anyhow::Result;
use async_trait::async_trait;
use crate::setup::constants::netbird;
use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

pub struct InstallService;

#[async_trait]
impl Task for InstallService {
    fn description(&self) -> String {
        String::from("NetBird - Install Service")
    }
    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        let output = Command::new("systemctl")
            .arg("cat")
            .arg("netbird")
            .output()?;

        if output.status.success() {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }
    async fn make_present(&self) -> Result<Success> {
        let netbird = netbird::unpacked_executable()?;

        let _ = Command::new(netbird)
            .arg("service")
            .arg("install")
            .evaluate_requiring_success()?;

        Ok(Success::default())
    }
}
