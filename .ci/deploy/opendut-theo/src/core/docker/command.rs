use std::process::{Command, Output, Stdio};
use std::ffi::OsStr;
use std::path::PathBuf;
use anyhow::{anyhow, Error};
use tracing::{trace};
use std::io;
use crate::core::command_ext::TheoCommandExtensions;
use crate::core::docker::checks;
use crate::core::localenv::LOCALENV_SECRETS_ENV_FILE;
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
            .arg(format!("name={name}"))
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
            .arg("--file")
            .arg(format!("./.ci/deploy/testenv/{compose_dir}/docker-compose.yml"))
            .arg("--env-file")
            .arg(".env-theo")
            .arg("--env-file")
            .arg(".env")
    }

    pub(crate) fn add_common_project_env(&mut self) -> &mut Self {
        self.arg("--env-file")
            .arg(".env")
    }

    pub(crate) fn add_localenv_args(&mut self) -> &mut Self {
        self.arg("compose")
            .arg("--file")
            .arg("./.ci/deploy/localenv/docker-compose.yml")
            .arg("--env-file")
            .arg("./.ci/deploy/localenv/.env.development")
    }

    pub(crate) fn add_localenv_secrets_args(&mut self) -> &mut Self {
        self.arg("--env-file")
            .arg(LOCALENV_SECRETS_ENV_FILE)
    }

    pub(crate) fn expect_output(&mut self, error_message: &str) -> Result<Output, anyhow::Error> {
        self.debug_log_executed_command();
        let result = self.command.output();
        match result {
            Ok(output) => { Ok(output) }
            Err(error) => {
                Err(anyhow!(TheoError::DockerCommandFailed(format!("Failed to execute docker command. {error_message}\nCause: {error}"))))
            }
        }
    }

    pub(crate) fn debug_log_executed_command(&mut self) -> &mut Self {
        let program = self.command.get_program().to_string_lossy();
        let args = self.command.get_args()
            .map(|a| a.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        trace!("Executing docker command: '{} {}'", program, args);
        self
    }

    pub(crate) fn expect_show_status(&mut self, error_message: &str) -> Result<i32, Error> {
        self.expect_status(error_message, true)
    }

    pub(crate) fn expect_status(&mut self, error_message: &str, show_output: bool) -> Result<i32, Error> {
        self.debug_log_executed_command();
        if !show_output {
            self.command
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        let command_status = self
            .command
            .status()
            .map_err(|cause| anyhow!(TheoError::DockerCommandFailed(format!("{error_message}. Cause: {cause}"))))?;

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
