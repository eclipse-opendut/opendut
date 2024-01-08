use crate::docker::{check_docker_compose_is_installed, docker_compose_down, DockerCoreServices};
use crate::project::project_root_dir;

pub(crate) fn stop_testenv() {
    // prerequisites
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_down(DockerCoreServices::Keycloak.as_str(), false);
    docker_compose_down(DockerCoreServices::Carl.as_str(), false);
    docker_compose_down(DockerCoreServices::Netbird.as_str(), false);
    docker_compose_down(DockerCoreServices::Firefox.as_str(), false);
}
