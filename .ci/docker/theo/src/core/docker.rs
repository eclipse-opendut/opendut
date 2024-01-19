use std::{env, io};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::thread::sleep;
use std::time::Duration;

use anyhow::{anyhow, Error};

use crate::core::{OPENDUT_FIREFOX_EXPOSE_PORT, SLEEP_TIME_SECONDS, TheoError, TIMEOUT_SECONDS};
use crate::core::command_ext::TheoCommandExtensions;
use crate::core::project::ProjectRootDir;
use crate::core::util::consume_output;

pub(crate) enum DockerCoreServices {
    Network,
    Carl,
    CarlOnHost,
    Dev,
    Keycloak,
    Edgar,
    Netbird,
    Firefox,
}

impl DockerCoreServices {
    pub fn as_str(&self) -> &'static str {
        match self {
            DockerCoreServices::Carl => "carl",
            DockerCoreServices::CarlOnHost => "carl-on-host",
            DockerCoreServices::Dev => "dev",
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


pub struct DockerCommand {
    command: Command,
}

impl DockerCommand {
    pub(crate) fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.command.arg(arg);
        self
    }
    fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
        where
            K: AsRef<OsStr>,
            V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }

    /// Create docker command in project root directory.
    pub(crate) fn new() -> Self {
        let mut command = Command::new("docker");
        command.current_dir(PathBuf::project_dir());
        Self { command }
    }

    pub(crate) fn docker_checks(&self) -> crate::Result {
        check_docker_is_installed()?;
        check_docker_compose_is_installed()?;
        check_docker_daemon_communication()?;
        Ok(())
    }

    pub(crate) fn add_common_args(&mut self, compose_dir: &str) -> &mut Self {
        self.arg("compose")
            .arg("-f")
            .arg(format!("./.ci/docker/{}/docker-compose.yml", compose_dir))
            .arg("--env-file")
            .arg(".env-theo")
            .arg("--env-file")
            .arg(".env")
    }

    pub(crate) fn add_netbird_api_key_to_env(&mut self) -> Result<&mut Self, TheoError> {
        let netbird_api_key = get_netbird_api_key()?;
        self.env("NETBIRD_API_TOKEN", &netbird_api_key);
        Ok(self)
    }

    pub(crate) fn expect_output(&mut self, error_message: &str) -> Result<Output, anyhow::Error> {
        let result = self.command.output();
        match result {
            Ok(output) => { Ok(output) }
            Err(error) => {
                Err(anyhow!(TheoError::DockerCommandFailed(format!("Failed to execute docker command. {}\nCause: {}", error_message, error))))
            }
        }
    }

    pub(crate) fn expect_status(&mut self, error_message: &str) -> Result<i32, anyhow::Error> {
        let command_status = self
            .command
            .status()
            .map_err(|cause| anyhow!(TheoError::DockerCommandFailed(format!("{}. Cause: {}", error_message, cause))))?;

        if command_status.success() {
            Ok(command_status.code().unwrap_or(1))
        } else {
            Err(anyhow!(TheoError::DockerCommandFailed(error_message.to_string())))
        }
    }

    pub(crate) fn output(&mut self) -> io::Result<Output> {
        self.command.output()
    }

    pub(crate) fn run(&mut self) -> crate::Result {
        self.command
            .run();
        Ok(())
    }

    pub(crate) fn check_output_status(output: Result<Output, Error>) -> Result<bool, Error> {
        match output {
            Ok(output) => {
                Ok(output.status.code().unwrap_or(1) == 0)
            }
            Err(error) => {
                Err(error)
            }
        }
    }
}

fn check_docker_is_installed() -> Result<Output, Error> {
    DockerCommand::new()
        .arg("version")
        .expect_output("Failed to run docker version. Check if docker is installed.")
}

fn check_docker_compose_is_installed() -> Result<Output, Error> {
    DockerCommand::new()
        .arg("compose")
        .arg("version")
        .expect_output("Failed to run docker compose. Check if docker compose plugin is installed.")
}

fn check_docker_daemon_communication() -> Result<Output, Error> {
    DockerCommand::new()
        .arg("ps")
        .expect_output("Failed to communicate with docker daemon. Check privileges, e.g. membership of the 'docker' group.")
}

pub(crate) fn docker_compose_build(compose_dir: &str) -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(compose_dir)
        .arg("build")
        .expect_status(format!("Failed to execute docker compose build for directory: {}.", compose_dir).as_str())
}

pub(crate) fn docker_compose_up(compose_dir: &str) -> Result<i32, Error> {
    DockerCommand::new()
        .add_common_args(compose_dir)
        .arg("up")
        .arg("-d")
        .expect_status(format!("Failed to execute docker compose up for directory: {}.", compose_dir).as_str())
}


pub(crate) fn docker_compose_down(compose_dir: &str, delete_volumes: bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.add_common_args(compose_dir);
    if delete_volumes {
        command.arg("down").arg("--volumes");
    } else {
        command.arg("down");
    }
    command.expect_status(format!("Failed to execute docker compose down for directory: {}.", compose_dir).as_str())
}

pub(crate) fn docker_compose_network_create() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("compose")
        .arg("-f")
        .arg(format!("./.ci/docker/{}/docker-compose.yml", DockerCoreServices::Network))
        .arg("up")
        .arg("--force-recreate")
        .expect_status("Failed to create docker network.")
}

pub(crate) fn docker_compose_network_delete() -> Result<i32, Error> {
    DockerCommand::new()
        .arg("network")
        .arg("rm")
        .arg("opendut_network")
        .expect_status("Failed to create docker network.")
}

pub(crate) fn wait_for_netbird_api_key() -> crate::Result {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start = std::time::Instant::now();
    while !check_netbird_api_key_available()? {
        if start.elapsed() > timeout {
            return Err(anyhow!(
                TheoError::Timeout(String::from("Timeout while waiting for netbird api key to be available."))
            ));
        }
        println!("Waiting for netbird api key to be available...");
        sleep(Duration::from_secs(SLEEP_TIME_SECONDS));
    }
    Ok(())
}

pub(crate) fn wait_for_keycloak_provisioned() -> crate::Result {
    let timeout = Duration::from_secs(TIMEOUT_SECONDS);
    let start = std::time::Instant::now();
    while !check_keycloak_provisioning_done()? {
        if start.elapsed() > timeout {
            return Err(anyhow!(
                TheoError::Timeout(String::from("While waiting for keycloak provisioning to be done."))
            ));
        }
        println!("Waiting for keycloak to be provisioned...");
        sleep(Duration::from_secs(SLEEP_TIME_SECONDS));
    }
    Ok(())
}


fn check_netbird_api_key_available() -> Result<bool, Error> {
    let command_output = DockerCommand::new()
        .add_common_args(DockerCoreServices::Netbird.as_str())
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("management_init")
        .arg("ls")
        .arg("/management/api_key")
        .expect_output("Failed to check if netbird api key is available");
    DockerCommand::check_output_status(command_output)
}

fn check_keycloak_provisioning_done() -> Result<bool, Error> {
    let command_output = DockerCommand::new()
        .add_common_args(DockerCoreServices::Keycloak.as_str())
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("init_keycloak")
        .arg("ls")
        .arg("/opt/keycloak/data/provisioned")
        .expect_output("Failed to check if keycloak was provisioned");
    DockerCommand::check_output_status(command_output)
}

pub fn get_netbird_api_key() -> Result<String, TheoError> {
    let mut command = DockerCommand::new();
    let command_status = command
        .add_common_args(DockerCoreServices::Netbird.as_str())
        .arg("run")
        .arg("--entrypoint")
        .arg("")
        .arg("--rm")
        .arg("management_init")
        .arg("cat")
        .arg("/management/api_key")
        .output();

    consume_output(command_status).map_err(|cause| TheoError::DockerCommandFailed(format!("Failed to get netbird api key from netbird_management_init container. Error: {}", cause)))
}

pub fn start_opendut_firefox_container(expose: &bool) -> crate::Result {
    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("-f")
        .arg(".ci/docker/firefox/docker-compose.yml");

    let expose_env_value = env::var(OPENDUT_FIREFOX_EXPOSE_PORT).unwrap_or("false".to_string()).eq("true");
    if *expose || expose_env_value {
        command.arg("-f")
            .arg(".ci/docker/firefox/expose_ports.yml")
    } else {
        command.arg("-f")
            .arg(".ci/docker/firefox/localhost_ports.yml")
    };
    command.arg("--env-file")
        .arg(".env-theo")
        .arg("--env-file")
        .arg(".env")
        .arg("up")
        .arg("-d")
        .expect_status("Failed to execute docker compose command for firefox.")?;
    Ok(())
}

pub fn start_netbird(expose: &bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("-f")
        .arg(".ci/docker/netbird/docker-compose.yml");

    let expose_env_value = env::var(OPENDUT_FIREFOX_EXPOSE_PORT).unwrap_or("false".to_string()).eq("true");
    if *expose || expose_env_value {
        command.arg("-f")
            .arg(".ci/docker/netbird/expose_ports.yml");
    };
    command.arg("--env-file")
        .arg(".env-theo")
        .arg("--env-file")
        .arg(".env")
        .arg("up")
        .arg("-d")
        .expect_status("Failed to execute compose command for netbird")
}

