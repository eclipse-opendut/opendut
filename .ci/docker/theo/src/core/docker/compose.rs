use std::env;

use anyhow::Error;

use crate::core::docker::{DockerCommand, DockerCoreServices};
use crate::core::OPENDUT_EXPOSE_PORTS;

pub(crate) fn docker_compose_build(compose_dir: &str) -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(compose_dir)
        .arg("build")
        .expect_status(format!("Failed to execute docker compose build for directory: {}.", compose_dir).as_str())
}

pub fn docker_compose_up_expose_ports(compose_dir: &str, expose: &bool) -> crate::Result {
    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("-f")
        .arg(format!(".ci/docker/{}/docker-compose.yml", compose_dir));

    let expose_env_value = env::var(OPENDUT_EXPOSE_PORTS).unwrap_or("false".to_string()).eq("true");
    if *expose || expose_env_value {
        command.arg("-f")
            .arg(format!(".ci/docker/{}/expose_ports.yml", compose_dir))
    } else {
        command.arg("-f")
            .arg(format!(".ci/docker/{}/localhost_ports.yml", compose_dir))
    };
    command.arg("--env-file")
        .arg(".env-theo")
        .arg("--env-file")
        .arg(".env")
        .arg("up")
        .arg("-d")
        .expect_status(&*format!("Failed to execute docker compose command for {}.", compose_dir))?;
    Ok(())
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
        .expect_status("Failed to delete docker network.")
}
