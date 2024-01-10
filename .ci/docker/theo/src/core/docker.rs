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
    fn docker_checks();
    fn add_common_args(&mut self, compose_dir: &str) -> &mut Self;
    fn add_netbird_api_key_to_env(&mut self) -> &mut Self;
}

impl DockerCommand for Command {
    fn docker() -> Command {
        Command::new("docker")
    }

    fn docker_checks() {
        check_docker_compose_is_installed();
        check_docker_daemon_communication();
    }

    fn add_common_args(&mut self, compose_dir: &str) -> &mut Self {
        self.arg("compose")
            .arg("-f")
            .arg(format!("./.ci/docker/{}/docker-compose.yml", compose_dir))
            .arg("--env-file")
            .arg(".env-theo")
            .arg("--env-file")
            .arg(".env")
    }

    fn add_netbird_api_key_to_env(&mut self) -> &mut Self {
        let netbird_api_key = get_netbird_api_key();
        self.env("NETBIRD_API_TOKEN", &netbird_api_key)
    }
}


fn check_docker_compose_is_installed() {
    Command::docker()
        .arg("compose")
        .arg("version")
        .output()
        .expect("Failed to run docker compose. Check if docker compose plugin is installed.");
}

fn check_docker_daemon_communication() {
    Command::docker()
        .arg("ps")
        .output()
        .expect("Failed to communicate with docker daemon. Check privileges, e.g. membership of the 'docker' group.");
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

pub(crate) fn wait_for_netbird_api_key() {
    let timeout = std::time::Duration::from_secs(120);
    let start = std::time::Instant::now();
    while !check_netbird_api_key_available() {
        if start.elapsed() > timeout {
            panic!("Timeout while waiting for netbird api key to be available.");
        }
        println!("Waiting for netbird api key to be available...");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

fn check_netbird_api_key_available() -> bool {
    let mut command = Command::docker();
    command.add_common_args(DockerCoreServices::Netbird.as_str());
    let command_status = command
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("management_init")
        .arg("ls")
        .arg("/management/api_key")
        .current_dir(PathBuf::project_dir())
        .output().expect("Failed to check if netbird api key is available");

    command_status.status.code().unwrap_or(1) == 0
}

fn get_netbird_api_key() -> String {
    let mut command = Command::docker();
    command.add_common_args(DockerCoreServices::Netbird.as_str());
    let command_status = command
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("management_init")
        .arg("cat")
        .arg("/management/api_key")
        .current_dir(PathBuf::project_dir())
        .output();

    consume_output(command_status).expect("Failed to get netbird api key from netbird_management_init container")
}
