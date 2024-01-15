use std::path::PathBuf;
use std::process::Command;

use crate::core::docker::{docker_compose_build, docker_compose_down, DockerCommand, DockerCoreServices};
use crate::core::project::ProjectRootDir;

#[derive(Debug, clap::Parser)]
pub struct TestEdgarCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Start edgar cluster.")]
    Start,
    #[command(about = "Stop edgar cluster.")]
    Stop,
    #[command(about = "Build edgar.")]
    Build,
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
            TaskCli::Build => {
                docker_compose_build(DockerCoreServices::Edgar.as_str());
            }
        }
    }
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
    docker_compose_build(DockerCoreServices::Edgar.as_str());
    start_edgar_in_docker();
}
