use crate::commands::edgar::TestEdgarCli;
use crate::core::dist::make_distribution_if_not_present;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::compose::{docker_compose_down, docker_compose_up_expose_ports};
use crate::core::docker::localenv::{delete_localenv_secrets, docker_localenv_shutdown, TestenvCarlImage, LOCALENV_SECRETS_ENV_FILE, LOCALENV_SECRETS_PATH, LOCALENV_TELEMETRY_ENABLED};
use crate::core::docker::services::DockerCoreServices;
use crate::core::docker::show_error_if_unhealthy_containers_were_found;
use crate::core::project::{load_theo_environment_variables, ProjectRootDir};
use anyhow::Error;
use clap::ArgAction;
use std::path::PathBuf;
use tracing::debug;

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
    /// Provision secrets for localenv.
    Provision,
    /// Start test environment.
    #[command(alias = "up")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_EXPOSE_PORTS=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,

        #[arg(long, action = ArgAction::SetTrue)]
        skip_firefox: bool,

        /// Skip telemetry (set OPENDUT_LOCALENV_TELEMETRY_ENABLED=0)
        #[arg(long, action = ArgAction::SetTrue)]
        skip_telemetry: bool,
    },
    /// Stop test environment.
    Stop,
    /// Show Docker network.
    Network,
    /// Destroy test environment.
    Destroy,
    /// Run EDGAR cluster creation.
    Edgar(TestEdgarCli),
}

impl TestenvCli {
    pub(crate) fn default_handling(self) -> crate::Result {
        load_theo_environment_variables();
        DockerCommand::new().docker_checks()?;

        match self.task {
            TaskCli::Build => {
                make_distribution_if_not_present()?;
                Self::build_localenv_containers()?;
            }
            TaskCli::Provision => {
                Self::provision_localenv_secrets()?;
            }
            TaskCli::Start { expose, skip_firefox, skip_telemetry } => {
                if !PathBuf::project_path_buf().join(LOCALENV_SECRETS_ENV_FILE).exists() {
                    Self::build_localenv_containers()?;
                }

                start_localenv_in_docker(skip_telemetry)?;

                if !skip_firefox {
                    docker_compose_up_expose_ports(DockerCoreServices::Firefox.as_str(), expose)?;
                }

                show_error_if_unhealthy_containers_were_found()?;

                println!("Secrets for localenv are loaded from '{}'", LOCALENV_SECRETS_ENV_FILE);
                println!("Go to OpenDuT Browser at http://localhost:3000/");
            }
            TaskCli::Stop => {
                println!("Stopping localenv testenv...");
                docker_localenv_shutdown(false)?;
            }
            TaskCli::Network => {
                crate::core::network::docker_inspect_network()?;
                show_error_if_unhealthy_containers_were_found()?;
            }
            TaskCli::Destroy => {
                docker_compose_down(DockerCoreServices::Firefox.as_str(), true)?;
                docker_compose_down(DockerCoreServices::Edgar.as_str(), true)?;
                docker_localenv_shutdown(true)?;
                delete_localenv_secrets()?;
            }

            TaskCli::Edgar(cli) => {
                cli.default_handling()?;
            }
        }
        Ok(())
    }

    fn provision_localenv_secrets() -> Result<(), Error> {
        debug!("Provisioning secrets for localenv...");
        DockerCommand::new()
            .add_localenv_args()
            .arg("up")
            .arg("--build")
            .arg("provision-secrets")
            .expect_status("Failed to provision localenv secrets")?;

        delete_localenv_secrets()?;
        // copy secrets to host
        DockerCommand::new()
            .arg("cp")
            .arg("opendut-provision-secrets:/provision/")
            .arg(LOCALENV_SECRETS_PATH)
            .expect_status("Failed to copy localenv secrets.")?;
        debug!("Copied secrets to host at {}", LOCALENV_SECRETS_PATH);

        Ok(())
    }

    fn build_localenv_containers() -> Result<i32, Error> {
        Self::provision_localenv_secrets()?;
        debug!("Building localenv containers...");
        DockerCommand::new()
            .add_localenv_args()
            .add_common_project_env()
            .arg("build")
            .expect_status("Failed to build localenv services")
    }
}

fn start_localenv_in_docker(skip_telemetry: bool) -> Result<i32, Error> {
    let carl_image = TestenvCarlImage::create()?;
    let mut docker_command = DockerCommand::new();
    docker_command
        .add_localenv_args()
        .add_localenv_secrets_args()
        .arg("--file")
        .arg(".ci/deploy/testenv/carl-on-host/docker-compose.localenv.override.yml")  // dev mode
        .env("OPENDUT_DOCKER_IMAGE_HOST", carl_image.image_host)
        .env("OPENDUT_DOCKER_IMAGE_NAMESPACE", carl_image.namespace)
        .env("OPENDUT_CARL_IMAGE_VERSION", carl_image.tag);
    if skip_telemetry {
        debug!("Disabling telemetry for localenv in testenv mode.");
        docker_command.env(LOCALENV_TELEMETRY_ENABLED, "0");
    }
    docker_command
        .arg("up")
        .arg("--detach")
        .expect_status("Failed to start localenv for testenv")
}
