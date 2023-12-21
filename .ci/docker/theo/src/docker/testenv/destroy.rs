use crate::docker::{check_docker_compose_is_installed, docker_compose_down, docker_compose_network_delete, DockerCoreServices};
use crate::project::project_root_dir;

pub(crate) fn destroy_testenv() {
    // prerequisites
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_down(DockerCoreServices::Keycloak.as_str(), true);
    docker_compose_down(DockerCoreServices::Carl.as_str(), true);
    docker_compose_down(DockerCoreServices::Netbird.as_str(), true);
    docker_compose_down(DockerCoreServices::Firefox.as_str(), true);
    docker_compose_network_delete();
}
