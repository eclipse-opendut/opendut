use anyhow::Error;
use clap::ArgAction;

use crate::commands::vagrant::running_in_opendut_vm;
use crate::core::carl_config::CarlConfiguration;
use crate::core::docker::{show_error_if_unhealthy_containers_were_found, start_netbird};
use crate::core::docker::command::DockerCommand;
use crate::core::docker::compose::{docker_compose_build, docker_compose_down, docker_compose_network_create, docker_compose_up_expose_ports};
use crate::core::docker::services::DockerCoreServices;
use crate::core::project::load_theo_environment_variables;

/// Build and start development environment
#[derive(clap::Parser)]
pub struct DevCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Start dev containers.
    #[command(alias = "up")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_EXPOSE_PORTS=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,
    },
    /// Stop dev containers.
    Stop,
    /// Build dev container.
    Build,
    /// Shell in Dev container.
    Shell,
    /// CARL environment run configuration for your IDE (different output in VM and host!).
    CarlConfig,
    /// EDGAR container.
    EdgarShell,
}

impl DevCli {
    pub(crate) fn default_handling(&self) -> crate::Result {
        load_theo_environment_variables();
        DockerCommand::new().docker_checks()?;

        match self.task {
            TaskCli::Start { expose } => {
                docker_compose_network_create()?;

                println!("Starting services...");
                docker_compose_up_expose_ports(DockerCoreServices::Firefox.as_str(), expose)?;
                docker_compose_up_expose_ports(DockerCoreServices::Keycloak.as_str(), expose)?;
                crate::core::docker::keycloak::wait_for_keycloak_provisioned()?;
                start_netbird(expose)?;
                crate::core::docker::netbird::wait_for_netbird_api_key()?;

                println!("Stopping carl in container (if present).");
                docker_compose_down(DockerCoreServices::Carl.as_str(), false)?;

                start_carl_traefik_forwarder()?;

                show_error_if_unhealthy_containers_were_found()?;
            }
            TaskCli::Stop => {
                docker_compose_down(DockerCoreServices::CarlOnHost.as_str(), false)?;
                docker_compose_down(DockerCoreServices::Keycloak.as_str(), false)?;
                docker_compose_down(DockerCoreServices::Carl.as_str(), false)?;
                docker_compose_down(DockerCoreServices::Netbird.as_str(), false)?;
                docker_compose_down(DockerCoreServices::Firefox.as_str(), false)?;
            }
            TaskCli::Build => {
                docker_compose_build(DockerCoreServices::Dev.as_str())?;
            }
            TaskCli::Shell => {
                let mut command = DockerCommand::new();
                command.add_common_args(DockerCoreServices::Dev.as_str());

                command
                    .arg("run")
                    .arg("--rm")
                    .arg("--entrypoint=\"\"")
                    .arg("-it")
                    .arg("opendut_dev")
                    .arg("bash")
                    .expect_status("Failed to start shell in dev container.")?;
            }
            TaskCli::EdgarShell => {
                DockerCommand::new()
                    .add_common_args(DockerCoreServices::Edgar.as_str())
                    .arg("run")
                    .arg("--rm")
                    //.arg("--entrypoint=\"\"")
                    .arg("-it")
                    .arg("leader")
                    .arg("bash")
                    .run()?;
            }
            TaskCli::CarlConfig => {
                let netbird_api_key = crate::core::docker::netbird::get_netbird_api_key()?;
                let carl_config = if running_in_opendut_vm() {
                    println!("# Following configuration assumes services are running in virtual machine.");
                    CarlConfiguration::testenv_in_vm_config(netbird_api_key)
                } else {
                    println!("# Following configuration assumes services are running in docker on host.");
                    CarlConfiguration::testenv_on_host_config(netbird_api_key)
                };
                println!("{}", carl_config.config_toml());

            }
        }
        Ok(())
    }
}

fn start_carl_traefik_forwarder() -> Result<i32, Error> {
    println!("Starting traefik to forward data to carl running on host system.");

    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("--file")
        .arg(format!(".ci/docker/{}/docker-compose.yml", DockerCoreServices::CarlOnHost));

    if running_in_opendut_vm() {
        command.arg("--file").arg(format!(".ci/docker/{}/vm.yml", DockerCoreServices::CarlOnHost));
    } else {
        command.arg("--file").arg(format!(".ci/docker/{}/localhost.yml", DockerCoreServices::CarlOnHost));
    };
    command.arg("--env-file")
        .arg(".env-theo")
        .arg("--env-file")
        .arg(".env");

    command
        .arg("up")
        .arg("--detach")
        .expect_status("Failed to execute compose command for netbird traefik forwarder.")
}
