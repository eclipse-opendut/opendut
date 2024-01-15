use std::path::PathBuf;
use std::process::Command;

use clap::ArgAction;
use dotenvy::dotenv;

use crate::commands::edgar::TestEdgarCli;
use crate::core::dist::make_distribution_if_not_present;
use crate::core::docker::{docker_compose_build, docker_compose_down, docker_compose_network_create, docker_compose_network_delete, docker_compose_up, DockerCommand, DockerCoreServices, wait_for_netbird_api_key};
use crate::core::OPENDUT_THEO_DISABLE_ENV_CHECKS;
use crate::core::project::{boolean_env_var, check_dot_env_variables, check_user_provided_dot_env_variables, ProjectRootDir, write_theo_dynamic_env_vars};

/// Build and start test environment.
#[derive(Debug, clap::Parser)]
pub struct TestenvCli {
    #[command(subcommand)]
    pub(crate) task: TaskCli,
    /// Disable environment variable checks (OPENDUT_THEO_DISABLE_ENV_CHECKS=true)
    #[clap(long, short, action = ArgAction::SetTrue)]
    disable_env_checks: bool,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Build docker containers.")]
    Build,
    #[command(about = "Start test environment.")]
    Start,
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
        self.check_environment_variables();
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
            TaskCli::Start => {
                // prerequisites
                docker_compose_network_create();

                // start services
                docker_compose_up(DockerCoreServices::Firefox.as_str());
                docker_compose_up(DockerCoreServices::Keycloak.as_str());
                docker_compose_up(DockerCoreServices::Netbird.as_str());
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

    fn check_environment_variables(&self) {
        write_theo_dynamic_env_vars();
        let custom_env = PathBuf::project_path_buf().join(".env-theo");
        dotenvy::from_path(custom_env).expect(".env-theo file not found");

        // load environment variables from .env file
        match dotenv().is_ok() {
            true => {}
            false => {
                check_user_provided_dot_env_variables();
                println!("Deliberately exiting here to review changes to '.env' file.");
                return;
            }
        }
        check_user_provided_dot_env_variables();

        if self.disable_env_checks && !boolean_env_var(OPENDUT_THEO_DISABLE_ENV_CHECKS) {
            check_dot_env_variables();
        } else {
            println!("Skipping environment variable checks.");
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

