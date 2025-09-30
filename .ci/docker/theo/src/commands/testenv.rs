use anyhow::Error;
use clap::{ArgAction, Parser};
use std::path::PathBuf;
use strum::IntoEnumIterator;

use crate::commands::edgar::TestEdgarCli;
use crate::core::dist::make_distribution_if_not_present;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::compose::{docker_compose_build, docker_compose_down};
use crate::core::docker::localenv::{delete_localenv_secrets, docker_localenv_shutdown, LOCALENV_SECRETS_ENV_FILE, LOCALENV_SECRETS_PATH};
use crate::core::docker::services::DockerCoreServices;
use crate::core::docker::show_error_if_unhealthy_containers_were_found;
use crate::core::project::{load_theo_environment_variables, ProjectRootDir};

/// Build and start test environment.
#[derive(clap::Parser)]
pub struct TestenvCli {
    #[command(subcommand)]
    pub(crate) task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Build Docker containers.
    Build,
    /// Start test environment.
    #[command(alias = "up")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_EXPOSE_PORTS=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,

        #[arg(long, short, action = ArgAction::SetTrue)]
        skip_build: bool,

        #[arg(long, action = ArgAction::SetTrue)]
        skip_firefox: bool,

        #[arg(long, action = ArgAction::SetTrue)]
        skip_telemetry: bool,
    },
    /// Stop test environment.
    Stop,
    /// Show Docker network.
    Network,
    /// Destroy test environment.
    Destroy(DestroyArgs),
    /// Run EDGAR cluster creation.
    Edgar(TestEdgarCli),
}

#[derive(Parser, Debug)]
#[clap(version)]
pub struct DestroyArgs {
    #[clap(short = 's', long)]
    service: Option<DockerCoreServices>,
}

impl TestenvCli {
    pub(crate) fn default_handling(self) -> crate::Result {
        load_theo_environment_variables();
        DockerCommand::new().docker_checks()?;

        match self.task {
            TaskCli::Build => {
                Self::docker_compose_build_testenv_services()?;
            }
            TaskCli::Start { expose, skip_build, skip_firefox, skip_telemetry } => {
                // Check if localenv has been provisioned, TODO: might use docker volume instead?
                if !PathBuf::project_path_buf().join(LOCALENV_SECRETS_ENV_FILE).exists() {
                    Self::provision_and_build_localenv()?;
                }

                Self::start_localenv_for_testenv()?;
                // TODO: wait until localenv is fully started?
                show_error_if_unhealthy_containers_were_found()?;

                println!("Go to OpenDuT at https://carl.opendut.local/");
            }
            TaskCli::Stop => {
                println!("Stopping localenv testenv...");
                docker_localenv_shutdown(false)?;
            }
            TaskCli::Network => {
                crate::core::network::docker_inspect_network()?;
                show_error_if_unhealthy_containers_were_found()?;
            }
            TaskCli::Destroy(service) => {
                match &service.service {
                    Some(service) => {
                        match service {
                            DockerCoreServices::Carl => docker_compose_down(DockerCoreServices::Carl.as_str(), true)?,
                            DockerCoreServices::CarlOnHost => docker_compose_down(DockerCoreServices::CarlOnHost.as_str(), true)?,
                            DockerCoreServices::Edgar => docker_compose_down(DockerCoreServices::Edgar.as_str(), true)?,
                        };
                    }
                    None => {
                        println!("Destroying all services."); // omit docker network
                        for docker_service in DockerCoreServices::iter() {
                            docker_compose_down(docker_service.as_str(), true)?;
                        }
                    }

                }
                docker_localenv_shutdown(true)?;
                delete_localenv_secrets()?;
            }

            TaskCli::Edgar(cli) => {
                cli.default_handling()?;
            }
        }
        Ok(())
    }

    fn docker_compose_build_testenv_services() -> Result<(), Error> {
        make_distribution_if_not_present()?;

        docker_compose_build(DockerCoreServices::Carl.as_str())?;
        Ok(())
    }

    fn provision_and_build_localenv() -> Result<(), Error> {
        println!("Provisioning secrets for localenv...");

        // Provision secrets if needed
        DockerCommand::new()
            .add_localenv_args()
            .arg("up")
            .arg("provision-secrets")
            .expect_status("Failed to provision localenv secrets")?;

        // Build the localenv services
        DockerCommand::new()
            .add_localenv_args()
            .add_common_project_env()
            .arg("build")
            .expect_status("Failed to build localenv services")?;

        delete_localenv_secrets()?;
        // copy secrets to host
        DockerCommand::new()
            .arg("cp")
            .arg("opendut-provision-secrets")
            .arg("/provision/")
            .arg(LOCALENV_SECRETS_PATH)
            .expect_status("Successfully copied localenv secrets.")?;

        Ok(())
    }

    fn start_localenv_for_testenv() -> Result<(), Error> {
        println!("Starting localenv in testenv mode (telemetry disabled)...");

        DockerCommand::new()
            .add_localenv_args_with_disabled_telemetry()
            .add_localenv_secrets_args()
            .arg("up")
            .arg("--detach")
            .expect_status("Failed to start localenv for testenv")?;

        Ok(())
    }
}

fn start_carl_in_docker() -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(DockerCoreServices::Carl.as_str())
        .arg("up")
        .arg("--detach")
        .expect_status("Failed to execute compose command for CARL.")
}
