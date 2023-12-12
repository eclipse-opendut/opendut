use anyhow::{Context, Result};
use config::Config;
use futures::executor::block_on;

use crate::common;
use crate::common::settings;
use crate::setup::task::{Success, Task, TaskFulfilled};

pub struct CheckCarlReachable;
impl Task for CheckCarlReachable {
    fn description(&self) -> String {
        String::from("Check CARL Reachable")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        Ok(TaskFulfilled::Unchecked)
    }
    fn execute(&self) -> Result<Success> {
        let settings = settings::load_with_overrides(Config::default())?;

        let _ = block_on(common::carl::connect(&settings.config))
            .context("Failed to connect to CARL")?;
        Ok(Success::default())
    }
}
