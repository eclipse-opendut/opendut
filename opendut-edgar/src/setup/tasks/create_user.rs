use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use crate::setup::User;

use crate::setup::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

fn passwd_file() -> PathBuf { PathBuf::from("/etc/passwd") }

pub struct CreateUser {
    pub service_user: User,
}
impl Task for CreateUser {
    fn description(&self) -> String {
        format!("Create User \"{}\"", self.service_user.name)
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let passwd = fs::read_to_string(passwd_file())?;
        let user_exists = passwd.lines()
            .any(|line| line.starts_with(&self.service_user.name));
        if user_exists {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        Command::new("useradd")
            .arg("--no-create-home")
            .arg(&self.service_user.name)
            .evaluate_requiring_success()
            .context("Error when trying to create user '{USER_NAME}'.")?;
        Ok(Success::default())
    }
}
