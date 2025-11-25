use cicero::path::repo_path;
use tracing::info;

use std::path::PathBuf;

use crate::core::commands::TRUNK;
use crate::fs;

use crate::core::types::parsing::package::PackageSelection;
use crate::util::RunRequiringSuccess;
use crate::Package;

const PACKAGE: Package = Package::Lea;

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
    Run(crate::tasks::run::RunCli),
    Licenses(crate::tasks::licenses::LicensesCli),

    /// Compile and bundle LEA for distribution
    DistributionBuild(BuildCli),
}

#[derive(clap::Args)]
pub struct BuildCli {
    #[arg(raw = true)]
    passthrough: Vec<String>
}

impl LeaCli {
    #[tracing::instrument(name="lea", skip(self))]
    pub fn default_handling(self) -> crate::Result {
        match self.task {
            TaskCli::Build(BuildCli { passthrough }) => {
                let release_build = false;
                build::build(release_build, passthrough)?
            },
            TaskCli::Run(cli) => run::run(cli.passthrough)?,
            TaskCli::Licenses(cli) => cli.default_handling(PackageSelection::Single(PACKAGE))?,
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
    pub fn build(release_build: bool, passthrough: Vec<String>) -> crate::Result {
        build_impl(release_build, passthrough, out_dir())
    }

    pub fn out_dir() -> PathBuf {
        self_dir().join("dist")
    }
}

pub mod distribution_build {
    use super::*;

    #[tracing::instrument]
    pub fn distribution_build(passthrough: Vec<String>) -> crate::Result {
        let release = true;
        build_impl(release, passthrough, out_dir())
    }

    pub fn out_dir() -> PathBuf {
        crate::constants::target_dir().join("lea").join("distribution")
    }
}

pub mod run {
    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn run(passthrough: Vec<String>) -> crate::Result {
        info!("Starting LEA. You can view the web-UI at: https://localhost:8080");

        TRUNK.command()
            .arg("watch")
            .args(passthrough)
            .current_dir(self_dir())
            .run_requiring_success()?;
        Ok(())
    }
}

pub fn self_dir() -> PathBuf {
    repo_path!("opendut-lea/")
}

fn build_impl(release: bool, passthrough: Vec<String>, out_dir: PathBuf) -> crate::Result {
    let working_dir = self_dir();

    fs::create_dir_all(&out_dir)?;

    let mut command = TRUNK.command();
    command.arg("build");

    if release {
        command.arg("--release");
    }

    command.arg("--dist").arg(&out_dir);

    command
        .args(passthrough)
        .current_dir(working_dir)
        .run_requiring_success()?;

    info!("Placed distribution into: {}", out_dir.display());

    Ok(())
}
