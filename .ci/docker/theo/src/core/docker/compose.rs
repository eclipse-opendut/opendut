use anyhow::Error;
use crate::core::docker::{DockerCommand, DockerCoreServices};

pub(crate) fn docker_compose_build(compose_dir: &str) -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(compose_dir)
        .arg("build")
        .expect_status(format!("Failed to execute docker compose build for directory: {}.", compose_dir).as_str())
}

pub(crate) fn docker_compose_up(compose_dir: &str) -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(compose_dir)
        .arg("up")
        .arg("-d")
        .expect_status(format!("Failed to execute docker compose up for directory: {}.", compose_dir).as_str())
}


pub(crate) fn docker_compose_down(compose_dir: &str, delete_volumes: bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.add_common_args(compose_dir);
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    command.expect_status(format!("Failed to execute docker compose down for directory: {}.", compose_dir).as_str())
}

pub(crate) fn docker_compose_network_create() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("compose")
        .arg("-f")
        .arg(format!("./.ci/docker/{}/docker-compose.yml", DockerCoreServices::Network))
        .arg("up")
        .arg("--force-recreate")
        .expect_status("Failed to create docker network.")
}

pub(crate) fn docker_compose_network_delete() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("network")
        .arg("rm")
        .arg("opendut_network")
        .expect_status("Failed to create docker network.")
}
