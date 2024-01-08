use std::process::Command;

use anyhow::{Context, Result};

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

pub struct CheckOsRequirements;
impl Task for CheckOsRequirements {
    fn description(&self) -> String {
        String::from("Check OS Requirements")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {

        Command::new("systemctl")
            .arg("--version")
            .evaluate_requiring_success()
            .context("SystemD is required.")?;

        Ok(Success::default())
    }
}
