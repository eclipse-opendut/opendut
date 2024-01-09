use std::path::PathBuf;
use std::process::Command;

use crate::core::docker::{check_docker_compose_is_installed, docker_compose_network_create, docker_compose_up, DockerCommand, DockerCoreServices, wait_for_netbird_api_key};
use crate::core::project::ProjectRootDir;

fn start_carl_in_docker() {
    let mut command = Command::docker();
    command.add_common_args(DockerCoreServices::Carl.as_str());
    command.add_netbird_api_key_to_env();

    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(PathBuf::project_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for edgar. {}", cause));
    assert!(command_status.success());

}

pub(crate) fn start_testenv() {
    // prerequisites
    println!("git project root: {}", PathBuf::project_dir());
    check_docker_compose_is_installed();
    docker_compose_network_create();

    // start services
    docker_compose_up(DockerCoreServices::Firefox.as_str());
    docker_compose_up(DockerCoreServices::Keycloak.as_str());
    docker_compose_up(DockerCoreServices::Netbird.as_str());
    wait_for_netbird_api_key();
    start_carl_in_docker();


    // TODO: start edgar requires additional steps to run in managed mode
    println!("Go to OpenDuT Browser at http://localhost:3000/")
}
