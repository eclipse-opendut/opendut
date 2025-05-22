use anyhow::{Context, Result};
use async_trait::async_trait;
use config::Config;

use crate::common;
use crate::common::settings;
use crate::common::task::{Success, Task, TaskStateFulfilled};

pub struct CheckCarlReachable;

#[async_trait]
impl Task for CheckCarlReachable {
    fn description(&self) -> String {
        String::from("Check CARL Reachable")
    }
    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        Ok(TaskStateFulfilled::Unchecked)
    }
    async fn make_present(&self) -> Result<Success> {
        let settings = settings::load_with_overrides(Config::default())?;

        let _ = common::carl::connect(&settings.config).await
            .context("Failed to connect to CARL")?;
        Ok(Success::default())
    }
}
