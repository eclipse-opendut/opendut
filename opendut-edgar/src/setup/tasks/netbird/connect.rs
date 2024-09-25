use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use tracing::debug;
use url::Url;

use opendut_types::vpn::netbird::SetupKey;

use crate::common::task::{Success, Task, TaskFulfilled};

const UP_CHECK_RETRIES: usize = 50;
const UP_CHECK_INTERVAL: Duration = Duration::from_millis(200);

pub struct Connect {
    pub management_url: Url,
    pub setup_key: SetupKey,
    pub mtu: u16,
}

#[async_trait]
impl Task for Connect {
    fn description(&self) -> String {
        String::from("NetBird - Connect")
    }
    async fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let mut client = opendut_netbird_client_api::client::Client::connect().await?;
        let is_up = client.check_is_up().await?;
        if is_up {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    async fn execute(&self) -> Result<Success> {
        let mut client = opendut_netbird_client_api::client::Client::connect().await?;

        client.login(&self.setup_key, &self.management_url, self.mtu).await
            .context("Error during NetBird-Login")?;

        client.up().await
            .context("Error during NetBird-Up")?;

        for _ in 1..=UP_CHECK_RETRIES {
            let is_up = client.check_is_up().await?;
            if is_up {
                debug!("NetBird Client reports that it is logged in and up.");
                return Ok(Success::default())
            }
            tokio::time::sleep(UP_CHECK_INTERVAL).await
        }
        Err(anyhow!("Connection to NetBird Management Service at '{}' was not up after {}*{} ms.", self.management_url, UP_CHECK_RETRIES, UP_CHECK_INTERVAL.as_millis()))
    }
}
