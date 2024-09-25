use crate::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use async_trait::async_trait;
use crate::setup::User;

use crate::common::task::{Success, Task, TaskFulfilled};
use crate::setup::util::EvaluateRequiringSuccess;

fn passwd_file() -> PathBuf { PathBuf::from("/etc/passwd") }

pub struct CreateUser {
    pub service_user: User,
}

#[async_trait]
impl Task for CreateUser {
    fn description(&self) -> String {
        format!("Create User \"{}\"", self.service_user.name)
    }

    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let passwd = fs::read_to_string(passwd_file())?;

        let user_exists = passwd.lines()
            .any(|line| match line.split(':').next() {
                Some(user) => user == self.service_user.name,
                None => false,
            });

        if user_exists {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }

    async fn execute(&self) -> Result<Success> {
        Command::new("useradd")
            .arg("--no-create-home")
            .arg(&self.service_user.name)
            .evaluate_requiring_success()
            .context(format!("Error when trying to create user '{}'.", self.service_user.name))?;

        Ok(Success::default())
    }
}
