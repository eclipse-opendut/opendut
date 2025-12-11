use std::ops::Not;
use std::process;
use std::process::Command;
use std::time::Duration;

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use tracing::{debug, info};
use url::Url;

use opendut_model::vpn::netbird::SetupKey;

use crate::common::task::{Success, Task, TaskStateFulfilled};
use crate::setup::constants;

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
    async fn check_present(&self) -> Result<TaskStateFulfilled> {
        let mut client = opendut_netbird_client_api::client::Client::connect().await?;
        let is_up = client.check_is_up().await?;
        if is_up {
            Ok(TaskStateFulfilled::Yes)
        } else {
            Ok(TaskStateFulfilled::No)
        }
    }
    async fn make_present(&self) -> Result<Success> {

        {
            let process::Output { status, stdout, stderr } =
                Command::new(constants::netbird::unpacked_executable()?.as_os_str())
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
                return Ok(Success::default())
            }
            tokio::time::sleep(UP_CHECK_INTERVAL).await
        }
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


fn add_fields_into_json_object_string(
    fields: Vec<(&str, serde_json::Value)>,
    config: &str,
) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(config)?;

    for (key, value) in fields.into_iter() {
        json[key] = value;
    }
    Ok(json.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_insert_field_into_json_object() -> Result<()> {
        let config = sample_config();
        let field = vec![
            ("field1", json!("value1")),
            ("field2", json!(1234)),
        ];

        let result = add_fields_into_json_object_string(field, &config)?;

        assert_eq!(
            result,
            json!({
                "WgIface": "wt0",
                "WgIfaceMtu": 1280,
                "WgPort": 51820,
                "field1": "value1",
                "field2": 1234,
            }).to_string()
        );

        Ok(())
    }


    /// Snippet with similar structure to NetBird config.json
    fn sample_config() -> String {
        json!({
            "WgIface": "wt0",
            "WgIfaceMtu": 1280,
            "WgPort": 51820,
        })
        .to_string()
    }
}
