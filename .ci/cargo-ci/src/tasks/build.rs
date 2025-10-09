use std::path::PathBuf;

use crate::{constants, Package};
use crate::core::commands::CROSS;
use crate::types::Arch;
use crate::core::types::parsing::target::TargetSelection;
use crate::util::RunRequiringSuccess;


/// Perform a release build, without bundling a distribution.
#[derive(Debug, clap::Parser)]
#[command(hide=true)]
pub struct DistributionBuildCli {
    #[arg(long, default_value_t)]
    pub target: TargetSelection,

    /// Build artifacts in release mode, with optimizations
    #[arg(short='r', long="release")]
    pub release_build: bool,
}

#[tracing::instrument(skip_all)]
pub fn distribution_build(package: Package, target: Arch, release_build: bool) -> crate::Result {
    let mut command = CROSS.command();

    command
        .arg("--package").arg(package.ident())
        .arg("--target-dir").arg(cross_target_dir().as_os_str()) //explicitly set target-base-dir to fix unreliable caching behavior
        .arg("--target").arg(target.triple());

    if release_build || option_env!("CI").is_some() { //always perform release builds in CI runner
        command.arg("--release");
    }

    command.run_requiring_success()
}

pub fn out_dir(package: Package, target: Arch) -> PathBuf {
    cross_target_dir().join(target.triple()).join("release").join(package.ident())
}

fn cross_target_dir() -> PathBuf {
    constants::target_dir().join("cross")
}
