use std::env;
use std::path::PathBuf;
use std::process::Command;

use clap::ArgAction;
use crate::commands::vagrant::running_in_opendut_vm;

use crate::core::docker::{docker_compose_build, docker_compose_down, docker_compose_network_create, docker_compose_up, DockerCommand, DockerCoreServices, get_netbird_api_key, start_netbird, start_opendut_firefox_container, wait_for_netbird_api_key};
use crate::core::OPENDUT_FIREFOX_EXPOSE_PORT;
use crate::core::project::{load_theo_environment_variables, ProjectRootDir};

#[derive(Debug, clap::Parser)]
pub struct DevCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Start dev containers.")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_FIREFOX_EXPOSE_PORT=true
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
    pub(crate) fn default_handling(&self) {
        load_theo_environment_variables();
        Command::docker_checks();

        match self.task {
            TaskCli::Start { expose } => {
                docker_compose_network_create();

                println!("Starting services...");
                start_opendut_firefox_container(&expose);
                docker_compose_up(DockerCoreServices::Keycloak.as_str());
                start_netbird(&expose);
                wait_for_netbird_api_key();

                println!("Stopping carl in container (if present).");
                docker_compose_down(DockerCoreServices::Carl.as_str(), false);

                start_carl_traefik_forwarder();
            }
            TaskCli::Stop => {
                docker_compose_down(DockerCoreServices::CarlOnHost.as_str(), false);
                docker_compose_down(DockerCoreServices::Keycloak.as_str(), false);
                docker_compose_down(DockerCoreServices::Carl.as_str(), false);
                docker_compose_down(DockerCoreServices::Netbird.as_str(), false);
                docker_compose_down(DockerCoreServices::Firefox.as_str(), false);
            }
            TaskCli::Build => {
                docker_compose_build(DockerCoreServices::Dev.as_str());
            }
            TaskCli::Shell => {
                let mut command = Command::docker();
                command.add_common_args(DockerCoreServices::Dev.as_str());
                command.add_netbird_api_key_to_env();

                command
                    .arg("run")
                    .arg("--rm")
                    .arg("--entrypoint=\"\"")
                    .arg("-it")
                    .arg("opendut_dev")
                    .arg("bash")
                    .current_dir(PathBuf::project_dir())
                    .run();
            }
            TaskCli::EdgarShell => {
                let mut command = Command::docker();
                command.add_common_args(DockerCoreServices::Edgar.as_str());
                command.add_netbird_api_key_to_env();

                command
                    .arg("run")
                    .arg("--rm")
                    .arg("--entrypoint=\"\"")
                    .arg("-it")
                    .arg("router")
                    .arg("bash")
                    .current_dir(PathBuf::project_dir())
                    .run();
            }
            TaskCli::CarlConfig => {
                let netbird_api_key = get_netbird_api_key();
                let netbird_management_ip = if running_in_opendut_vm() {
                    "192.168.56.10"
                } else {
                    "192.168.32.211"
                };

                println!("
# These are the environment variables to run CARL in your IDE of choice.

OPENDUT_CARL_NETWORK_REMOTE_HOST=carl
OPENDUT_CARL_NETWORK_REMOTE_PORT=443
OPENDUT_CARL_VPN_ENABLED=true
OPENDUT_CARL_VPN_KIND=netbird
OPENDUT_CARL_VPN_NETBIRD_URL=http://{}/api
OPENDUT_CARL_VPN_NETBIRD_HTTPS_ONLY=false
OPENDUT_CARL_VPN_NETBIRD_AUTH_SECRET={}
OPENDUT_CARL_VPN_NETBIRD_AUTH_TYPE=personal-access-token
OPENDUT_CARL_VPN_NETBIRD_AUTH_HEADER=Authorization"
                         ,
                         netbird_management_ip,
                         netbird_api_key
                );
            }
        }
    }
}

fn start_carl_traefik_forwarder() {
    println!("Starting traefik to forward data to carl running on host system.");

    let mut command = Command::docker();
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

    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(PathBuf::project_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for netbird: {}", cause));
    assert!(command_status.success());

}
