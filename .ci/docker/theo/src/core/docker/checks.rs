use std::process::Output;
use anyhow::Error;
use crate::core::docker::DockerCommand;

pub(crate) fn check_docker_is_installed() -> Result<Output, Error> {
    DockerCommand::new()
        .arg("version")
        .expect_output("Failed to run docker version. Check if docker is installed.")
}

pub(crate) fn check_docker_compose_is_installed() -> Result<Output, Error> {
    DockerCommand::new()
        .arg("compose")
        .arg("version")
        .expect_output("Failed to run docker compose. Check if docker compose plugin is installed.")
}

pub(crate) fn check_docker_daemon_communication() -> Result<Output, Error> {
    DockerCommand::new()
        .arg("ps")
        .expect_output("Failed to communicate with docker daemon. Check privileges, e.g. membership of the 'docker' group.")
}
