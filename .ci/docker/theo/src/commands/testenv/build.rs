use std::path::PathBuf;
use crate::core::docker::{check_docker_compose_is_installed, docker_compose_build, docker_compose_network_create, DockerCoreServices};
use crate::core::project::ProjectRootDir;


pub(crate) fn build_testenv() {
    println!("git project root: {}", PathBuf::project_dir());
    check_docker_compose_is_installed();
    docker_compose_network_create();
    docker_compose_build(DockerCoreServices::Firefox.as_str());
    docker_compose_build(DockerCoreServices::Keycloak.as_str());
    docker_compose_build(DockerCoreServices::Carl.as_str());
    docker_compose_build(DockerCoreServices::Netbird.as_str());

}
