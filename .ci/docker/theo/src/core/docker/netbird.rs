use std::thread::sleep;
use std::time::Duration;
use anyhow::{anyhow, Error};
use crate::core::{SLEEP_TIME_SECONDS, TheoError, TIMEOUT_SECONDS};
use crate::core::docker::{DockerCommand, DockerCoreServices};
use crate::core::util::consume_output;

pub(crate) fn wait_for_netbird_api_key() -> crate::Result {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start = std::time::Instant::now();
    while !check_netbird_api_key_available()? {
        if start.elapsed() > timeout {
            return Err(anyhow!(
                TheoError::Timeout(String::from("Timeout while waiting for netbird api key to be available."))
            ));
        }
        println!("Waiting for netbird api key to be available...");
        sleep(Duration::from_secs(SLEEP_TIME_SECONDS));
    }
    Ok(())
}

fn check_netbird_api_key_available() -> Result<bool, Error> {
    let command_output = DockerCommand::new()
        .add_common_args(DockerCoreServices::Netbird.as_str())
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("management_init")
        .arg("ls")
        .arg("/management/api_key")
        .expect_output("Failed to check if netbird api key is available");
    DockerCommand::check_output_status(command_output)
}


pub(crate) fn get_netbird_api_key() -> Result<String, TheoError> {
    let mut command = DockerCommand::new();
    let command_status = command
        .add_common_args(DockerCoreServices::Netbird.as_str())
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("management_init")
        .arg("cat")
        .arg("/management/api_key")
        .output();

    consume_output(command_status).map_err(|cause| TheoError::DockerCommandFailed(format!("Failed to get netbird api key from netbird_management_init container. Error: {}", cause)))
}
