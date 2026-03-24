use std::ops::Not;
use std::process;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use tracing::{debug, info};
use url::Url;

use opendut_model::vpn::netbird::SetupKey;
use opendut_util::settings::LoadedConfig;
use crate::common::settings::netbird::NetbirdClientConfig;
use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::service::vpn::VpnProcess;
use crate::setup::constants;

const UP_CHECK_RETRIES: usize = 50;
const UP_CHECK_INTERVAL: Duration = Duration::from_millis(200);

pub struct Connect {
    pub management_url: Url,
    pub setup_key: SetupKey,
    pub mtu: u16,
    pub config: Arc<LoadedConfig>,
}

#[async_trait]
impl Task for Connect {
    fn description(&self) -> String {
        String::from("NetBird - Connect")
    }

    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        Ok(TaskStateFulfilled::Unchecked) //do unconditionally, since we need to spawn the NetBird process to be able to check whether it is up
    }

    async fn make_present(&self) -> Result<Success> {

        let netbird_config = NetbirdClientConfig::load_from_config(&self.config)?;
        let netbird = VpnProcess::spawn_as_netbird(netbird_config).await?; //temporarily spawn NetBird process to be able to trigger its login routine

        {
            let process::Output { status, stdout, stderr } =
                Command::new(constants::netbird::unpacked_executable()?.as_os_str())
                    .env("SSL_CERT_FILE", constants::default_os_cert_store_ca_certificate_path())
                    .arg("up")
                    .arg("--management-url").arg(self.management_url.as_str())
                    .arg("--setup-key").arg(&self.setup_key.value)
                    .arg("--mtu").arg(self.mtu.to_string())
                    .output()?;

            let message = format_command_output(stdout, stderr)?;

            if status.success() {
                info!("Successfully ran `netbird up` command: {message}");
            } else {
                bail!("Error while running `netbird up` command: {message}");
            }
        }


        let mut client = opendut_netbird_client_api::client::Client::connect().await?;

        for _ in 1..=UP_CHECK_RETRIES {
            let is_up = client.check_is_up().await?;
            if is_up {
                debug!("NetBird Client reports that it is logged in and up.");

                netbird.terminate().await?; //only needed during Setup to login; will be started anew in EDGAR Service
                return Ok(Success::default())
            }
            tokio::time::sleep(UP_CHECK_INTERVAL).await
        }

        netbird.terminate().await?;
        Err(anyhow!("Connection to NetBird Management Service at '{}' was not up after {}*{} ms.", self.management_url, UP_CHECK_RETRIES, UP_CHECK_INTERVAL.as_millis()))
    }
}

fn format_command_output(stdout: Vec<u8>, stderr: Vec<u8>) -> Result<String> {
    let mut result = String::new();

    if stdout.is_empty().not() {
        let stdout = String::from_utf8(stdout)?;
        result.push_str("\nstdout:\n");
        result.push_str(&stdout);
    }
    if stderr.is_empty().not() {
        let stderr = String::from_utf8(stderr)?;
        result.push_str("\nstderr:\n");
        result.push_str(&stderr);
    }
    Ok(result)
}
