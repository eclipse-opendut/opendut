use crate::docker::{check_docker_compose_is_installed, docker_compose_network_create, docker_compose_up, DockerCoreServices};
use crate::project::project_root_dir;

pub(crate) fn start_testenv() {
    // prerequisites
    println!("git project root: {}", project_root_dir());
    check_docker_compose_is_installed();
    docker_compose_network_create();

    // start services
    docker_compose_up(DockerCoreServices::Firefox.as_str());
    docker_compose_up(DockerCoreServices::Keycloak.as_str());
    docker_compose_up(DockerCoreServices::Carl.as_str());
    docker_compose_up(DockerCoreServices::Netbird.as_str());

    // TODO: start edgar requires additional steps to run in managed mode
    println!("Go to OpenDuT Browser at http://localhost:3000/")
}
