use std::process::Command;

use anyhow::{Context, Result};

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::evaluate_requiring_success;

pub struct CheckOsRequirements;
impl Task for CheckOsRequirements {
    fn description(&self) -> String {
        String::from("Check OS Requirements")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {

        let mut systemd = Command::new("systemctl");
        let systemd = systemd.arg("--version");
        evaluate_requiring_success(systemd)
            .context("SystemD is required.")?;

        Ok(Success::default())
    }
}
