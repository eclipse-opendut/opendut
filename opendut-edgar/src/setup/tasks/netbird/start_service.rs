use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};

use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

const UP_CHECK_RETRIES: usize = 50;
const UP_CHECK_INTERVAL: Duration = Duration::from_millis(200);

pub struct StartService;
impl Task for StartService {
    fn description(&self) -> String {
        String::from("NetBird - Start Service")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let output = Command::new("systemctl")
            .arg("is-active")
            .arg("netbird")
            .output()?;

        if output.status.success() {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        let _ = Command::new("systemctl")
            .arg("start")
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
