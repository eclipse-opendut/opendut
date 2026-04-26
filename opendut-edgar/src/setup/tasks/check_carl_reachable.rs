use crate::common::settings;
use crate::common::task::{Success, Task, TaskStateFulfilled};
use anyhow::{Context, Result};
use async_trait::async_trait;
use config::Config;
use tokio_util::sync::CancellationToken;
use crate::service::start::ConnectAndStart;

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

        tokio::task::spawn_blocking(move || {
            let _ =
                tokio::runtime::Handle::current().block_on(
                    crate::service::start::connect_and_start(
                        &ConnectAndStart::CarlClient { out: None },
                        &settings,
                        CancellationToken::new(), //unused here
                    )
                )
                .context("Failed to connect to CARL")?;

            anyhow::Ok(())
        }).await??;


        Ok(Success::default())
    }
}
