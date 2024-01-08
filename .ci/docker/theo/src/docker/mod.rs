pub(crate) mod testenv;
pub(crate) mod network;

use std::process::{Command};
use crate::project::project_root_dir;

enum DockerCoreServices {
    Network,
    Carl,
    Keycloak,
    Edgar,
    Netbird,
    Firefox,
}

impl DockerCoreServices {
    fn as_str(&self) -> &'static str {
        match self {
            DockerCoreServices::Carl => "carl",
            DockerCoreServices::Keycloak => "keycloak",
            DockerCoreServices::Edgar => "edgar",
            DockerCoreServices::Netbird => "netbird",
            DockerCoreServices::Network => "network",
            DockerCoreServices::Firefox => "firefox",
        }
    }
}

fn check_docker_compose_is_installed() {
    let output = Command::new("docker")
        .arg("compose")
        .arg("version")
        .status()
        .expect("Failed to run docker compose. Check if docker compose plugin is installed. \
                See https://docs.docker.com/compose/install/linux/ for instructions.");
    assert!(output.success());
}

/// Run docker compose up from project root for compose-config-file in given directory.
///
/// Example:
///
///   docker compose -f ./.ci/docker/<compose_dir>/docker-compose.yml --env-file .env up -d
fn add_common_docker_args(command: &mut Command, compose_dir: &str) {
    command
        .arg("compose")
        .arg("-f")
        .arg(format!("./.ci/docker/{}/docker-compose.yml", compose_dir))
        .arg("--env-file")
        .arg(".env");
}

fn docker_compose_build(compose_dir: &str) {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, compose_dir);
    let command_status = command
        .arg("build")
        .current_dir(project_root_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute docker compose build for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}

fn docker_compose_up(compose_dir: &str) {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, compose_dir);
    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(project_root_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}


fn docker_compose_down(compose_dir: &str, delete_volumes: bool) {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, compose_dir);
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    let command_status = command
        .current_dir(project_root_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}

fn docker_compose_network_create() {
    let output = Command::new("docker")
        .arg("compose")
        .arg("-f")
        .arg(format!("./.ci/docker/{}/docker-compose.yml", DockerCoreServices::Network.as_str()))
        .arg("up")
        .arg("--force-recreate")
        .current_dir(project_root_dir())
        .status()
        .expect("Failed to create docker network.");

    assert!(output.success());
}

fn docker_compose_network_delete() {
    let output = Command::new("docker")
        .arg("network")
        .arg("rm")
        .arg("opendut_network")
        .status()
        .expect("Failed to create docker network.");

    assert!(output.success());
}

