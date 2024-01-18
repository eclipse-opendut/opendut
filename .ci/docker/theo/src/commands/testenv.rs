use std::path::PathBuf;
use std::process::Command;

use clap::ArgAction;

use crate::commands::edgar::TestEdgarCli;
use crate::core::dist::make_distribution_if_not_present;
use crate::core::docker::{docker_compose_build, docker_compose_down, docker_compose_network_create, docker_compose_network_delete, docker_compose_up, DockerCommand, DockerCoreServices, start_netbird, start_opendut_firefox_container, wait_for_netbird_api_key};
use crate::core::project::{load_theo_environment_variables, ProjectRootDir};

/// Build and start test environment.
#[derive(Debug, clap::Parser)]
pub struct TestenvCli {
    #[command(subcommand)]
    pub(crate) task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Build docker containers.")]
    Build,
    #[command(about = "Start test environment.")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_FIREFOX_EXPOSE_PORT=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,
    },
    #[command(about = "Stop test environment.")]
    Stop,
    #[command(about = "Show docker network.")]
    Network,
    #[command(about = "Destroy test environment.")]
    Destroy,
    #[command(about = "Run edgar cluster creation.")]
    Edgar(TestEdgarCli),
}

impl TestenvCli {
    pub(crate) fn default_handling(&self) {
        load_theo_environment_variables();
        Command::docker_checks();

        match &self.task {
            TaskCli::Build => {
                make_distribution_if_not_present();

                docker_compose_network_create();
                docker_compose_build(DockerCoreServices::Firefox.as_str());
                docker_compose_build(DockerCoreServices::Keycloak.as_str());
                docker_compose_build(DockerCoreServices::Carl.as_str());
                docker_compose_build(DockerCoreServices::Netbird.as_str());
            }
            TaskCli::Start { expose } => {
                // prerequisites
                docker_compose_network_create();

                // start services
                start_opendut_firefox_container(expose);
                docker_compose_up(DockerCoreServices::Keycloak.as_str());
                start_netbird(&expose);
                wait_for_netbird_api_key();
                start_carl_in_docker();


                println!("Go to OpenDuT Browser at http://localhost:3000/")
            }
            TaskCli::Stop => {
                docker_compose_down(DockerCoreServices::Keycloak.as_str(), false);
                docker_compose_down(DockerCoreServices::Carl.as_str(), false);
                docker_compose_down(DockerCoreServices::Netbird.as_str(), false);
                docker_compose_down(DockerCoreServices::Firefox.as_str(), false);
            }
            TaskCli::Network => {
                crate::core::network::docker_inspect_network();
            }
            TaskCli::Destroy => {
                docker_compose_down(DockerCoreServices::Firefox.as_str(), true);
                docker_compose_down(DockerCoreServices::Carl.as_str(), true);
                docker_compose_down(DockerCoreServices::Netbird.as_str(), true);
                docker_compose_down(DockerCoreServices::Keycloak.as_str(), true);
                docker_compose_network_delete();
            }
            TaskCli::Edgar(cli) => {
                cli.default_handling();
            }
        }
    }
}

fn start_carl_in_docker() {
    let mut command = Command::docker();
    command.add_common_args(DockerCoreServices::Carl.as_str());
    command.add_netbird_api_key_to_env();

    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(PathBuf::project_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for edgar. {}", cause));
    assert!(command_status.success());
}

