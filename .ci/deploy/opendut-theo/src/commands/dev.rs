use clap::ArgAction;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{anyhow, Context};
use home::home_dir;
use crate::core::carl_config::CarlConfiguration;
use crate::core::docker::command::DockerCommand;
use crate::core::docker::services::DockerCoreServices;
use crate::core::command_ext::TheoCommandExtensions;
use crate::core::project::{load_theo_environment_variables, ProjectRootDir};
use crate::core::{localenv, TestenvMode};

/// Build and start development environment
#[derive(clap::Parser)]
pub struct DevCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Start localenv in dev mode (CARL container forwards to host).
    #[command(alias = "up")]
    Start {
        /// Expose firefox container port (3000), or set OPENDUT_EXPOSE_PORTS=true
        #[arg(long, short, action = ArgAction::SetTrue)]
        expose: bool,
    },
    /// Start CARL with developer configuration.
    Carl,
    /// CARL environment run configuration for your IDE (different output in VM and host!).
    CarlConfig,
    /// EDGAR container.
    EdgarShell,
}

impl DevCli {
    pub(crate) fn default_handling(&self) -> crate::Result {
        load_theo_environment_variables();
        DockerCommand::new().docker_checks()?;

        let dev_mode = TestenvMode::CarlDeveloperIDE;

        match self.task {
            TaskCli::Start { expose } => {
                localenv::start(false, false, expose, &dev_mode)?;
            }
            TaskCli::Carl => {
                // Create CARL developer config
                let carl_config = CarlConfiguration::generate();
                let config_path = carl_temporary_config_path();
                fs::write(&config_path, carl_config.config_toml())
                    .context("Failed to write carl config to temporary location.")?;

                let carl_config_path = config_path.into_os_string().into_string()
                    .map_err(|_| anyhow!("Failed to convert config path to string."))?;

                let localenv_root_ca = PathBuf::project_path_buf()
                    .join(".ci/deploy/localenv/data/secrets/pki/opendut-ca.pem");
                Command::new("cargo")
                    .current_dir(PathBuf::project_path_buf())
                    .env("OPENDUT_CARL_CUSTOM_CONFIG_PATH", carl_config_path)
                    .env("SSL_CERT_FILE", &localenv_root_ca)
                    .arg("carl")
                    .run_requiring_success()?;
            }
            TaskCli::EdgarShell => {
                DockerCommand::new()
                    .add_common_args(DockerCoreServices::Edgar.as_str())
                    .arg("run")
                    .arg("--rm")
                    .arg("-it")
                    .arg("leader")
                    .arg("bash")
                    .run()?;
            }
            TaskCli::CarlConfig => {
                let carl_config = CarlConfiguration::generate();
                println!("{}", carl_config.config_toml());
            }
        }
        Ok(())
    }
}

fn carl_temporary_config_path() -> PathBuf {
    let config = home_dir().map(|path| path.join(".cache").join("opendut/carl/config.toml")).unwrap();
    if let Some(parent) = config.parent() {
        fs::create_dir_all(parent).expect("Failed to create carl config directory.");
    }
    config
}