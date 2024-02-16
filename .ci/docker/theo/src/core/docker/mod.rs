use std::{env, io};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::{Command, Output};

use anyhow::{anyhow, Error};
use serde::Serialize;
use strum::EnumIter;

use crate::core::{OPENDUT_EXPOSE_PORTS, TheoError};
use crate::core::command_ext::TheoCommandExtensions;
use crate::core::docker::netbird::get_netbird_api_key;
use crate::core::project::ProjectRootDir;

mod checks;
pub(crate) mod compose;

pub(crate) mod keycloak;
pub(crate) mod netbird;

#[derive(Debug, Clone, clap::ValueEnum, Default, Serialize, EnumIter)]
pub(crate) enum DockerCoreServices {
    Network,
    Carl,
    CarlOnHost,
    Dev,
    Keycloak,
    Edgar,
    Netbird,
    Firefox,
    #[default]
    All,
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
            DockerCoreServices::All => "all",
        }
    }
}

impl std::fmt::Display for DockerCoreServices {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


#[derive(Debug)]
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

    pub(crate) fn exists(name: &str) -> bool {
        let result = DockerCommand::new()
            .arg("ps")
            .arg("--quiet")
            .arg("--filter")
            .arg(format!("name={}", name))
            .expect_output("Failed to check if docker container exists.")
            .map_err(|_error| false);
        match result {
            Ok(output) => {
                let stdout = String::from_utf8(output.stdout).expect("Failed to convert docker output to string.");
                output.status.success() && !stdout.is_empty()
            }
            Err(_) => { false }
        }
    }

    pub(crate) fn new_exec(container_name: &str) -> Self {
        let mut cmd = Self::new();
        cmd.arg("exec").arg("--tty").arg(container_name);
        cmd
    }

    pub(crate) fn docker_checks(&self) -> crate::Result {
        checks::check_docker_is_installed()?;
        checks::check_docker_compose_is_installed()?;
        checks::check_docker_daemon_communication()?;
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

    pub(crate) fn enumerate_unhealthy_containers() -> Result<Vec<String>, Error> {
        let output = DockerCommand::new()
            .arg("ps")
            .arg("--filter")
            .arg("health=unhealthy")
            .arg("--format")
            .arg("{{.Names}}")
            .output()?;
        let stdout = String::from_utf8(output.stdout)?;
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}

/// The environment variable is managed by THEO.
/// Within the VM it is set to true, outside the VM it defaults to false.
/// Be careful when overriding this value.
pub fn determine_if_ports_shall_be_exposed(user_intents_to_expose: bool) -> bool {
    let expose_env_value = env::var(OPENDUT_EXPOSE_PORTS).unwrap_or("false".to_string()).eq("true");
    user_intents_to_expose || expose_env_value
}


pub fn show_error_if_unhealthy_containers_were_found() -> Result<(), Error> {
    let unhealthy_containers = DockerCommand::enumerate_unhealthy_containers()?;
    if unhealthy_containers.len() > 0 {
        println!("# Unhealthy containers: {:?}", unhealthy_containers);
        return Err(TheoError::UnhealthyContainersFound(format!("Found unhealthy docker containers: {:?}", unhealthy_containers)).into());
    } else {
        println!("# No unhealthy containers found.");
        Ok(())
    }
}

pub fn start_netbird(expose: &bool) -> Result<i32, Error> {
    let mut command = DockerCommand::new();
    command.arg("compose")
        .arg("-f")
        .arg(".ci/docker/netbird/docker-compose.yml");

    if determine_if_ports_shall_be_exposed(*expose) {
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

