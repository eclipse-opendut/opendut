use cicero::path::repo_path;
use tracing::info;

use std::path::PathBuf;

use crate::core::commands::TRUNK;
use crate::{fs, workspace};

use crate::core::types::parsing::package::PackageSelection;
use crate::Package;
use crate::tasks::build::BuildArgs;

use cicero::command_exit_ok::CommandExitOk;


const SELF_PACKAGE: &Package = &workspace::package::opendut_lea;

/// Tasks available or specific for LEA
#[derive(clap::Parser)]
#[command(alias="opendut-lea")]
pub struct LeaCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    /// Compile and bundle LEA for development
    Build(BuildCli),
    /// Start a development server which watches for file changes.
    Run(BuildCli),
    Licenses(crate::tasks::licenses::LicensesCli),

    /// Compile and bundle LEA for distribution
    DistributionBuild(BuildCli),
}

#[derive(clap::Args)]
pub struct BuildCli {
    /// Additional parameters to pass through to the started program
    #[arg(raw = true)]
    passthrough: Vec<String>
}

impl LeaCli {
    #[tracing::instrument(name="lea", skip(self))]
    pub fn run(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::Build(BuildCli { passthrough }) => {
                let build_args = BuildArgs {
                    release_build: false,
                    passthrough,
                };
                build::build(&build_args)?
            },
            TaskCli::Run(BuildCli { passthrough }) => run::run(passthrough)?,
            TaskCli::Licenses(cli) => cli.run(PackageSelection::Single(SELF_PACKAGE.clone()))?,
            TaskCli::DistributionBuild(BuildCli { passthrough }) => {
                distribution_build::distribution_build(passthrough)?
            },
        };
        Ok(())
    }
}

pub mod build {
    use super::*;

    #[tracing::instrument]
    pub fn build(build_args: &BuildArgs) -> anyhow::Result<()> {
        build_impl(build_args, out_dir())
    }

    pub fn out_dir() -> PathBuf {
        self_dir().join("dist")
    }
}

pub mod distribution_build {
    use super::*;

    #[tracing::instrument]
    pub fn distribution_build(passthrough: Vec<String>) -> anyhow::Result<()> {
        let build_args = BuildArgs {
            release_build: true,
            passthrough,
        };
        build_impl(&build_args, out_dir())
    }

    pub fn out_dir() -> PathBuf {
        crate::constants::target_dir().join("lea").join("distribution")
    }
}

pub mod run {
    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn run(passthrough: Vec<String>) -> anyhow::Result<()> {
        info!("Starting LEA. You can view the web-UI at: https://localhost:8080");

        TRUNK.command()
            .arg("watch")
            .args(passthrough)
            .current_dir(self_dir())
            .status_exit_ok()?;
        Ok(())
    }
}

pub fn self_dir() -> PathBuf {
    repo_path!("opendut-lea/")
}

fn build_impl(build_args: &BuildArgs, out_dir: PathBuf) -> anyhow::Result<()> {
    let BuildArgs { release_build, passthrough } = build_args;

    let working_dir = self_dir();

    fs::create_dir_all(&out_dir)?;

    let mut command = TRUNK.command();
    command.arg("build");

    if *release_build {
        command
            .arg("--release")
            .arg("--cargo-profile=wasm-release");
    }

    command.arg("--dist").arg(&out_dir);

    command
        .args(passthrough)
        .current_dir(working_dir)
        .status_exit_ok()?;

    info!("Placed distribution into: {}", out_dir.display());

    Ok(())
}
