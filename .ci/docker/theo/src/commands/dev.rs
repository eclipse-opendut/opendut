use anyhow::Error;
use clap::ArgAction;

use crate::commands::vagrant::running_in_opendut_vm;
use crate::core::carl_config::{print_carl_config_env, print_carl_config_toml};
use crate::core::docker::{DockerCommand, DockerCoreServices, start_netbird};
use crate::core::docker::compose::{docker_compose_build, docker_compose_down, docker_compose_network_create, docker_compose_up_expose_ports};
use crate::core::project::load_theo_environment_variables;

#[derive(Debug, clap::Parser)]
pub struct DevCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Start dev containers.", alias = "up")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_EXPOSE_PORTS=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,
    },
    #[command(about = "Stop dev containers.")]
    Stop,
    #[command(about = "Build dev container.")]
    Build,
    #[command(about = "Shell in Dev container.")]
    Shell,
    #[command(about = "Carl environment run configuration for your IDE.")]
    CarlConfig,
    #[command(about = "Edgar container.")]
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
                docker_compose_up_expose_ports(DockerCoreServices::Firefox.as_str(), &expose)?;
                docker_compose_up_expose_ports(DockerCoreServices::Keycloak.as_str(), &expose)?;
                crate::core::docker::keycloak::wait_for_keycloak_provisioned()?;
                start_netbird(&expose)?;
                crate::core::docker::netbird::wait_for_netbird_api_key()?;

                println!("Stopping carl in container (if present).");
                docker_compose_down(DockerCoreServices::Carl.as_str(), false)?;

                start_carl_traefik_forwarder()?;
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
                command.add_netbird_api_key_to_env()?;

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
                    .add_netbird_api_key_to_env()?
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
                let netbird_management_url = if running_in_opendut_vm() {
                    "http://192.168.56.10/api"
                } else {
                    "http://192.168.32.211/api"
                };

                print_carl_config_env(netbird_management_url, &netbird_api_key);
                print_carl_config_toml(netbird_management_url, &netbird_api_key);
            }
        }
        Ok(())
    }
}

fn start_carl_traefik_forwarder() -> Result<i32, Error> {
    println!("Starting traefik to forward data to carl running on host system.");

    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("-f")
        .arg(format!(".ci/docker/{}/docker-compose.yml", DockerCoreServices::CarlOnHost));

    if running_in_opendut_vm() {
        command.arg("-f").arg(format!(".ci/docker/{}/vm.yml", DockerCoreServices::CarlOnHost));
    } else {
        command.arg("-f").arg(format!(".ci/docker/{}/localhost.yml", DockerCoreServices::CarlOnHost));
    };
    command.arg("--env-file")
        .arg(".env-theo")
        .arg("--env-file")
        .arg(".env");

    command
        .arg("up")
        .arg("-d")
        .expect_status("Failed to execute compose command for netbird traefik forwarder.")
}
