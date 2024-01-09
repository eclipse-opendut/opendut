use std::path::PathBuf;
use std::process::Command;

use crate::core::docker::{check_docker_compose_is_installed, docker_compose_build, docker_compose_down, DockerCommand, DockerCoreServices};
use crate::core::project::ProjectRootDir;

#[derive(Debug, clap::Parser)]
pub struct TestEdgarCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

impl TestEdgarCli {
    pub(crate) fn default_handling(&self) {
        match self.task {
            TaskCli::Start => {
                run_edgar();
            }
            TaskCli::Stop => {
                docker_compose_down(DockerCoreServices::Edgar.as_str(), false);
            }
        }
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    Start,
    Stop,
}

pub(crate) fn build_edgar() {
    check_docker_compose_is_installed();
    docker_compose_build(DockerCoreServices::Edgar.as_str());
}


fn start_edgar_in_docker() {
    let mut command = Command::docker();
    command.add_common_args(DockerCoreServices::Edgar.as_str());
    command.add_netbird_api_key_to_env();

    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(PathBuf::project_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for edgar. {}", cause));
    assert!(command_status.success());

}

pub(crate) fn run_edgar() {
    check_docker_compose_is_installed();
    build_edgar();
    start_edgar_in_docker();
}
