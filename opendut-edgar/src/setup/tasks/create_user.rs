use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use crate::setup::constants::USER_NAME;

use crate::setup::task::{Success, Task, TaskFulfilled};

fn passwd_file() -> PathBuf { PathBuf::from("/etc/passwd") }

pub struct CreateUser;
impl Task for CreateUser {
    fn description(&self) -> String {
        format!("Create User \"{USER_NAME}\"")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let passwd = fs::read_to_string(passwd_file())?;
        let user_exists = passwd.lines()
            .any(|line| line.starts_with(USER_NAME));
        if user_exists {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        Command::new("useradd")
            .arg(USER_NAME)
            .status()
            .context("Error when trying to create user '{USER_NAME}'.")?;
        Ok(Success::default())
    }
}
