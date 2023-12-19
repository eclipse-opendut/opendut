use std::path::PathBuf;

use crate::{Arch, Package};
use crate::core::types::parsing::package::PackageSelection;

const PACKAGE: &Package = &Package::Cleo;


/// Tasks available or specific for CLEO
#[derive(Debug, clap::Parser)]
#[command(alias="opendut-cleo")]
pub struct CleoCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    Build(crate::tasks::build::BuildCli),
    Distribution(crate::tasks::distribution::DistributionCli),
    Licenses(crate::tasks::licenses::LicensesCli),
}

impl CleoCli {
    pub fn handle(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::Build(crate::tasks::build::BuildCli { target }) => {
                for target in target.iter() {
                    build::build_release(&target)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target }) => {
                for target in target.iter() {
                    distribution::cleo_distribution(&target)?;
                }
            }
            TaskCli::Licenses(implementation) => {
                implementation.handle(PackageSelection::Single(*PACKAGE))?;
            }
        };
        Ok(())
    }
}

pub mod build {
    use super::*;

    pub fn build_release(target: &Arch) -> anyhow::Result<()> {
        crate::tasks::build::build_release(PACKAGE, target)
    }
    pub fn out_dir(target: &Arch) -> PathBuf {
        crate::tasks::build::out_dir(PACKAGE, target)
    }
}

pub mod distribution {
    use super::*;

    #[tracing::instrument]
    pub fn cleo_distribution(target: &Arch) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        distribution::clean(PACKAGE, target)?;

        crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(PACKAGE, target)?;

        distribution::licenses::get_licenses(PACKAGE, target)?;

        distribution::bundle_collected_files(PACKAGE, target)?;

        Ok(())
    }
}
