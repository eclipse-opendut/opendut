use std::process::{Command, Output};
use std::ffi::OsStr;
use std::path::PathBuf;
use anyhow::{anyhow, Error};
use std::io;
use crate::core::command_ext::TheoCommandExtensions;
use crate::core::docker::checks;
use crate::core::docker::netbird::get_netbird_api_key;
use crate::core::project::ProjectRootDir;
use crate::core::TheoError;

#[derive(Debug)]
pub struct DockerCommand {
    command: Command,
}

impl DockerCommand {
    pub(crate) fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.command.arg(arg);
        self
    }
    pub(crate) fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
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

    pub(crate) fn container_exists(name: &str) -> bool {
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
