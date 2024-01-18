use std::path::PathBuf;
use std::process::Command;
use crate::core::docker::{docker_compose_build, docker_compose_down, docker_compose_network_create, docker_compose_up, DockerCommand, DockerCoreServices, get_netbird_api_key, wait_for_netbird_api_key};
use crate::core::project::{load_theo_environment_variables, ProjectRootDir};

#[derive(Debug, clap::Parser)]
pub struct DevCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    #[command(about = "Start dev containers.")]
    Start,
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
            TaskCli::Start => {
                docker_compose_network_create();

                println!("Starting services...");
                docker_compose_up(DockerCoreServices::Firefox.as_str());
                docker_compose_up(DockerCoreServices::Keycloak.as_str());
                docker_compose_up(DockerCoreServices::Netbird.as_str());
                wait_for_netbird_api_key();

                println!("Stopping carl in container (if present).");
                docker_compose_down(DockerCoreServices::Carl.as_str(), false);

                println!("Starting traefik to forward data to carl running on host system.");
                docker_compose_up(DockerCoreServices::CarlOnHost.as_str());
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

                println!("
# These are the environment variables to run CARL in your IDE of choice.

OPENDUT_CARL_NETWORK_REMOTE_HOST=carl
OPENDUT_CARL_NETWORK_REMOTE_PORT=443
OPENDUT_CARL_VPN_ENABLED=true
OPENDUT_CARL_VPN_KIND=netbird
OPENDUT_CARL_VPN_NETBIRD_URL=http://192.168.32.211/api
OPENDUT_CARL_VPN_NETBIRD_HTTPS_ONLY=false
OPENDUT_CARL_VPN_NETBIRD_AUTH_SECRET={}
OPENDUT_CARL_VPN_NETBIRD_AUTH_TYPE=personal-access-token
OPENDUT_CARL_VPN_NETBIRD_AUTH_HEADER=Authorization"
                         ,
                         netbird_api_key
                );
            }
        }
    }
}
