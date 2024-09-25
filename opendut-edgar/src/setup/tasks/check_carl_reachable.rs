use anyhow::{Context, Result};
use async_trait::async_trait;
use config::Config;

use crate::common;
use crate::common::settings;
use crate::common::task::{Success, Task, TaskFulfilled};

pub struct CheckCarlReachable;

#[async_trait]
impl Task for CheckCarlReachable {
    fn description(&self) -> String {
        String::from("Check CARL Reachable")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    async fn execute(&self) -> Result<Success> {
        let settings = settings::load_with_overrides(Config::default())?;

        let _ = common::carl::connect(&settings.config).await
            .context("Failed to connect to CARL")?;
        Ok(Success::default())
    }
}
