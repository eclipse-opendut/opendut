use crate::docker::{check_docker_compose_is_installed, docker_compose_build, docker_compose_network_create, DockerCoreServices};
use crate::project::project_root_dir;

pub(crate) fn build_testenv() {
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_network_create();
    docker_compose_build(DockerCoreServices::Firefox.as_str());
    docker_compose_build(DockerCoreServices::Keycloak.as_str());
    docker_compose_build(DockerCoreServices::Carl.as_str());
    docker_compose_build(DockerCoreServices::Netbird.as_str());

}
