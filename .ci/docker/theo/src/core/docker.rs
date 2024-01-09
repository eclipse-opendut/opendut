use std::path::PathBuf;
use std::process::Command;
use crate::core::project::ProjectRootDir;

use crate::core::util::consume_output;

pub(crate) enum DockerCoreServices {
    Network,
    Carl,
    Keycloak,
    Edgar,
    Netbird,
    Firefox,
}

impl DockerCoreServices {
    pub fn as_str(&self) -> &'static str {
        match self {
            DockerCoreServices::Carl => "carl",
            DockerCoreServices::Keycloak => "keycloak",
            DockerCoreServices::Edgar => "edgar",
            DockerCoreServices::Netbird => "netbird",
            DockerCoreServices::Network => "network",
            DockerCoreServices::Firefox => "firefox",
        }
    }
}

impl std::fmt::Display for DockerCoreServices {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub trait DockerCommand {
    fn docker() -> Command;
    fn add_common_args(&mut self, compose_dir: &str) -> &mut Self;
    fn add_netbird_api_key_to_env(&mut self) -> &mut Self;
}

impl DockerCommand for Command {
    fn docker() -> Command {
        Command::new("docker")
    }

    fn add_common_args(&mut self, compose_dir: &str) -> &mut Self {
        self.arg("compose")
            .arg("-f")
            .arg(format!("./.ci/docker/{}/docker-compose.yml", compose_dir))
            .arg("--env-file")
            .arg(".env")
    }

    fn add_netbird_api_key_to_env(&mut self) -> &mut Self {
        let netbird_api_key = get_netbird_api_key();
        self.env("NETBIRD_API_TOKEN", &netbird_api_key)
    }
}


pub(crate) fn check_docker_compose_is_installed() {
    let output = Command::docker()
        .arg("compose")
        .arg("version")
        .status()
        .expect("Failed to run docker compose. Check if docker compose plugin is installed. \
                See https://docs.docker.com/compose/install/linux/ for instructions.");
    assert!(output.success());
}

pub(crate) fn docker_compose_build(compose_dir: &str) {
    let mut command = Command::docker();
    command.add_common_args(compose_dir);
    let command_status = command
        .arg("build")
        .current_dir(PathBuf::project_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute docker compose build for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}

pub(crate) fn docker_compose_up(compose_dir: &str) {
    let mut command = Command::docker();
    command.add_common_args(compose_dir);
    let command_status = command
        .arg("up")
        .arg("-d")
        .current_dir(PathBuf::project_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}


pub(crate) fn docker_compose_down(compose_dir: &str, delete_volumes: bool) {
    let mut command = Command::docker();
    command.add_common_args(compose_dir);
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    let command_status = command
        .current_dir(PathBuf::project_dir())
        .status()
        .unwrap_or_else(|cause| panic!("Failed to execute compose command for directory: {}. {}", compose_dir, cause));

    assert!(command_status.success());
}

pub(crate) fn docker_compose_network_create() {
    let output = Command::docker()
        .arg("compose")
        .arg("-f")
        .arg(format!("./.ci/docker/{}/docker-compose.yml", DockerCoreServices::Network))
        .arg("up")
        .arg("--force-recreate")
        .current_dir(PathBuf::project_dir())
        .status()
        .expect("Failed to create docker network.");

    assert!(output.success());
}

pub(crate) fn docker_compose_network_delete() {
    let output = Command::docker()
        .arg("network")
        .arg("rm")
        .arg("opendut_network")
        .status()
        .expect("Failed to create docker network.");

    assert!(output.success());
}


fn get_netbird_api_key() -> String {
    let mut command = Command::docker();
    command.add_common_args(DockerCoreServices::Netbird.as_str());
    let command_status = command
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("management_init")
        .arg("cat")
        .arg("/management/api_key")
        .current_dir(PathBuf::project_dir())
        .output();

    consume_output(command_status).expect("Failed to get netbird api key from netbird_management_init container")
}
