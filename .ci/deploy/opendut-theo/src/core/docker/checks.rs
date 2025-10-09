use anyhow::Error;
use crate::core::docker::command::DockerCommand;

pub(crate) fn check_docker_is_installed() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("version")
        .expect_status("Failed to run docker version. Check if docker is installed.", false)
}

pub(crate) fn check_docker_compose_is_installed() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("compose")
        .arg("version")
        .expect_status("Failed to run docker compose. Check if docker compose plugin is installed.", false)
}

pub(crate) fn check_docker_daemon_communication() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("ps")
        .expect_status("Failed to communicate with docker daemon. Check privileges, e.g. membership of the 'docker' group.", false)
}
