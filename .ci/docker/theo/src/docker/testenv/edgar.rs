use std::process::Command;

use crate::docker::{add_common_docker_args, check_docker_compose_is_installed, docker_compose_build, docker_compose_down, DockerCoreServices};
use crate::project::project_root_dir;
use crate::util::consume_output;

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

pub(crate) fn get_netbird_api_key() -> String {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, DockerCoreServices::Netbird.as_str());
    let command_status = command
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("management_init")
        .arg("cat")
        .arg("/management/api_key")
        .current_dir(project_root_dir())
        .output();
    match command_status {
        Ok(_result) => {
            consume_output(_result)
        }
        Err(error) => {
            panic!("Failed to get netbird api key from netbird_management_init container: {:?}", error);
        }
    }

}

fn run_edgar_in_docker() {
    let mut command = Command::new("docker");
    add_common_docker_args(&mut command, DockerCoreServices::Edgar.as_str());
    let command = command
        .env("NETBIRD_API_TOKEN", get_netbird_api_key());

    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(project_root_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for edgar. {}", cause));
    assert!(command_status.success());

}

pub(crate) fn run_edgar() {
    check_docker_compose_is_installed();
    build_edgar();
    run_edgar_in_docker();
}
