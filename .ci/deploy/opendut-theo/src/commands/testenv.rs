use crate::commands::edgar::TestEdgarCli;
use crate::core::dist::make_distribution_if_not_present;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::compose::{docker_compose_down, docker_compose_up_expose_ports};
use crate::core::docker::localenv::{delete_localenv_secrets, docker_localenv_shutdown, TestenvCarlImage, LOCALENV_SECRETS_ENV_FILE, LOCALENV_SECRETS_PATH};
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
    /// Start test environment.
    #[command(alias = "up")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_EXPOSE_PORTS=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,

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
                Self::provision_and_build_localenv()?;
            }
            TaskCli::Start { expose, skip_firefox, skip_telemetry } => {
                // Check if localenv has been provisioned, TODO: consistent build/provision/start behavior
                if !PathBuf::project_path_buf().join(LOCALENV_SECRETS_ENV_FILE).exists() {
                    Self::provision_and_build_localenv()?;
                }

                let carl_image = build_carl_docker_image_with_special_tag_for_testenv()?;
                start_localenv_in_docker(skip_telemetry, carl_image)?;

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
            .arg("opendut-provision-secrets:/provision/")
            .arg(LOCALENV_SECRETS_PATH)
            .expect_status("Failed to copy localenv secrets.")?;
        debug!("Copied secrets to host at {}", LOCALENV_SECRETS_PATH);

        Ok(())
    }
}

fn start_localenv_in_docker(skip_telemetry: bool, carl_image: TestenvCarlImage) -> Result<i32, Error> {
    let mut docker_command = DockerCommand::new();
    docker_command
        .add_localenv_args()
        .add_localenv_secrets_args()
        .env("OPENDUT_DOCKER_IMAGE_HOST", carl_image.image_host)
        .env("OPENDUT_DOCKER_IMAGE_NAMESPACE", carl_image.namespace)
        .env("OPENDUT_CARL_IMAGE_VERSION", carl_image.tag);
    if skip_telemetry {
        debug!("Disabling telemetry for localenv in testenv mode.");
        docker_command.env("OPENDUT_LOCALENV_TELEMETRY_ENABLED", "0");
    }
    docker_command
        .arg("up")
        .arg("--detach")
        .expect_status("Failed to start localenv for testenv")
}


fn build_carl_docker_image_with_special_tag_for_testenv() -> Result<TestenvCarlImage, Error> {
    let carl_version = crate::core::metadata::get_package_version("opendut-carl");
    let carl_docker_file = PathBuf::project_path_buf().join(".ci/docker/carl/Dockerfile");
    let carl_image = TestenvCarlImage::new(&carl_version);

    DockerCommand::new().arg("build")
        .arg("-f")
        .arg(&carl_docker_file)
        .arg("--build-arg")
        .arg(format!("VERSION={}", carl_version))
        .arg("-t")
        .arg(carl_image.full_image_name())
        .arg(".")
        .expect_status("Failed to build docker image")?;


    Ok(carl_image)
}
