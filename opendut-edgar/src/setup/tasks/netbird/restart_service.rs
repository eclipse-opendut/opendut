use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

const UP_CHECK_RETRIES: usize = 50;
const UP_CHECK_INTERVAL: Duration = Duration::from_millis(200);

pub struct RestartService;

#[async_trait]
impl Task for RestartService {
    fn description(&self) -> String {
        String::from("NetBird - (Re-)Start Service")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    async fn execute(&self) -> Result<Success> {
        let _ = Command::new("systemctl")
            .arg("restart")
            .arg("netbird")
            .evaluate_requiring_success()?;

        let socket_path = opendut_netbird_client_api::client::socket_path();
        for _ in 1..=UP_CHECK_RETRIES {
            if socket_path.exists() {
                return Ok(Success::default())
            }
            thread::sleep(UP_CHECK_INTERVAL)
        }
        Err(anyhow!("NetBird Client Socket was not available {UP_CHECK_RETRIES}*{} ms after service start.", UP_CHECK_INTERVAL.as_millis()))
    }
}
