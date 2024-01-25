use std::thread::sleep;
use std::time::Duration;
use anyhow::{anyhow, Error};
use crate::core::{SLEEP_TIME_SECONDS, TheoError, TIMEOUT_SECONDS};
use crate::core::docker::{DockerCommand, DockerCoreServices};

pub(crate) fn wait_for_keycloak_provisioned() -> crate::Result {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start = std::time::Instant::now();
    while !check_keycloak_provisioning_done()? {
        if start.elapsed() > timeout {
            return Err(anyhow!(
                TheoError::Timeout(String::from("While waiting for keycloak provisioning to be done."))
            ));
        }
        println!("Waiting for keycloak to be provisioned...");
        sleep(Duration::from_secs(SLEEP_TIME_SECONDS));
    }
    Ok(())
}


fn check_keycloak_provisioning_done() -> Result<bool, Error> {
    let command_output = DockerCommand::new()
        .add_common_args(DockerCoreServices::Keycloak.as_str())
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("init_keycloak")
        .arg("ls")
        .arg("/opt/keycloak/data/provisioned")
        .expect_output("Failed to check if keycloak was provisioned");
    DockerCommand::check_output_status(command_output)
}
