use std::path::PathBuf;

use crate::core::docker::{docker_compose_down, DockerCoreServices};
use crate::core::project::ProjectRootDir;

pub(crate) fn stop_testenv() {
    // prerequisites
    println!("git project root: {}", PathBuf::project_dir());
    docker_compose_down(DockerCoreServices::Keycloak.as_str(), false);
    docker_compose_down(DockerCoreServices::Carl.as_str(), false);
    docker_compose_down(DockerCoreServices::Netbird.as_str(), false);
    docker_compose_down(DockerCoreServices::Firefox.as_str(), false);
}
