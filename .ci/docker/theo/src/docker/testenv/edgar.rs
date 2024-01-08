use crate::docker::{check_docker_compose_is_installed, docker_compose_build, docker_compose_up, DockerCoreServices};

pub(crate) fn build_edgar() {
    check_docker_compose_is_installed();
    docker_compose_build(DockerCoreServices::Edgar.as_str());
}

pub(crate) fn run_edgar() {
    check_docker_compose_is_installed();
    build_edgar();
    docker_compose_up(DockerCoreServices::Edgar.as_str());
}
