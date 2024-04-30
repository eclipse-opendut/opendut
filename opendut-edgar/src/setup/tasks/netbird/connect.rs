use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use futures::executor::block_on;
use tracing::debug;
use url::Url;

use opendut_types::vpn::netbird::SetupKey;

use crate::setup::task::{Success, Task, TaskFulfilled};

const UP_CHECK_RETRIES: usize = 50;
const UP_CHECK_INTERVAL: Duration = Duration::from_millis(200);

pub struct Connect {
    pub management_url: Url,
    pub setup_key: SetupKey,
    pub mtu: u16,
}
impl Task for Connect {
    fn description(&self) -> String {
        String::from("NetBird - Connect")
    }
    fn check_fulfilled(&self) -> Result<TaskFulfilled> {
        let mut client = block_on(opendut_netbird_client_api::client::Client::connect())?;
        let is_up = block_on(client.check_is_up())?;
        if is_up {
            Ok(TaskFulfilled::Yes)
        } else {
            Ok(TaskFulfilled::No)
        }
    }
    fn execute(&self) -> Result<Success> {
        let mut client = block_on(opendut_netbird_client_api::client::Client::connect())?;

        block_on(client.login(&self.setup_key, &self.management_url, self.mtu))
            .context("Error during NetBird-Login")?;

        block_on(client.up())
            .context("Error during NetBird-Up")?;

        for _ in 1..=UP_CHECK_RETRIES {
            let is_up = block_on(client.check_is_up())?;
            if is_up {
                debug!("NetBird Client reports that it is logged in and up.");
                return Ok(Success::default())
            }
            thread::sleep(UP_CHECK_INTERVAL)
        }
        Err(anyhow!("Connection to NetBird Management Service at '{}' was not up after {}*{} ms.", self.management_url, UP_CHECK_RETRIES, UP_CHECK_INTERVAL.as_millis()))
    }
}
