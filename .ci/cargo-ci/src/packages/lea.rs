use crate::fs;
use std::path::PathBuf;
use std::process::Command;
use repo_path::repo_path;
use tracing::info;
use crate::Arch;
use crate::core::dependency::Crate;

use crate::core::types::parsing::package::PackageSelection;
use crate::Package;
use crate::util::RunRequiringSuccess;

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
    Build,
    /// Start a development server which watches for file changes.
    Run(crate::tasks::run::RunCli),
    Licenses(crate::tasks::licenses::LicensesCli),

    /// Compile and bundle LEA for distribution
    DistributionBuild,
}

impl LeaCli {
    #[tracing::instrument(name="lea", skip(self))]
    pub fn default_handling(self) -> crate::Result {
        match self.task {
            TaskCli::Build => build::build()?,
            TaskCli::Run(cli) => run::run(cli.pass_through)?,
            TaskCli::Licenses(cli) => cli.default_handling(PackageSelection::Single(PACKAGE))?,
            TaskCli::DistributionBuild => distribution_build::distribution_build()?,
        };
        Ok(())
    }
}

pub mod build {
    use super::*;

    #[tracing::instrument]
    pub fn build() -> crate::Result {
        build_impl(out_dir())
    }

    pub fn out_dir() -> PathBuf {
        self_dir().join("dist")
    }
}

pub mod distribution_build {
    use super::*;

    #[tracing::instrument]
    pub fn distribution_build() -> crate::Result {
        build_impl(out_dir())
    }

    pub fn out_dir() -> PathBuf {
        crate::constants::target_dir().join("lea").join("distribution")
    }
}

pub mod run {
    use super::*;

    #[tracing::instrument]
    pub fn run(pass_through: Vec<String>) -> crate::Result {
        install_requirements()?;

        Command::new("trunk")
            .args([
                "watch",
                "--release",
            ])
            .args(pass_through)
            .current_dir(self_dir())
            .run_requiring_success()?;
        Ok(())
    }
}

#[tracing::instrument]
fn install_requirements() -> crate::Result {
    crate::util::install_toolchain(Arch::Wasm)?;

    crate::util::install_crate(Crate::Trunk)?;

    Ok(())
}

pub fn self_dir() -> PathBuf {
    repo_path!("opendut-lea/")
}

fn build_impl(out_dir: PathBuf) -> crate::Result {
    install_requirements()?;

    let working_dir = self_dir();

    fs::create_dir_all(&out_dir)?;

    Command::new("trunk")
        .args([
            "build",
            "--release",
            "--dist", &out_dir.display().to_string(),
        ])
        .current_dir(working_dir)
        .run_requiring_success()?;

    info!("Placed distribution into: {}", out_dir.display());

    Ok(())
}
