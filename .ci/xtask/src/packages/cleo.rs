use std::path::PathBuf;

use crate::{Target, Package};
use crate::core::types::parsing::package::PackageSelection;

const PACKAGE: Package = Package::Cleo;


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
    pub fn default_handling(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::Build(crate::tasks::build::BuildCli { target }) => {
                for target in target.iter() {
                    build::build_release(target)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target }) => {
                for target in target.iter() {
                    distribution::cleo_distribution(target)?;
                }
            }
            TaskCli::Licenses(implementation) => {
                implementation.default_handling(PackageSelection::Single(PACKAGE))?;
            }
        };
        Ok(())
    }
}

pub mod build {
    use super::*;

    pub fn build_release(target: Target) -> anyhow::Result<()> {
        crate::tasks::build::build_release(PACKAGE, target)
    }
    pub fn out_dir(target: Target) -> PathBuf {
        crate::tasks::build::out_dir(PACKAGE, target)
    }
}

pub mod distribution {
    use crate::tasks::distribution::copy_license_json::SkipGenerate;
    use super::*;

    #[tracing::instrument]
    pub fn cleo_distribution(target: Target) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        distribution::clean(PACKAGE, target)?;

        crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(PACKAGE, target)?;

        distribution::copy_license_json::copy_license_json(PACKAGE, target, SkipGenerate::No)?;

        distribution::bundle::bundle_files(PACKAGE, target)?;

        Ok(())
    }
}
