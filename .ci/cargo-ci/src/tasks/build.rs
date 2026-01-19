use std::ops::Not;
use std::path::PathBuf;

use cicero::distribution::build::Target;

use crate::{constants, Package};
use crate::core::commands::CROSS;
use crate::util::RunRequiringSuccess;


/// Perform a release build, without bundling a distribution.
#[derive(Debug, clap::Parser)]
#[command(hide=true)]
pub struct DistributionBuildCli {
    #[arg(long, default_value_t)]
    pub target: Target,

    /// Build artifacts in release mode, with optimizations
    #[arg(short='r', long="release")]
    pub release_build: bool,
}

#[tracing::instrument(skip_all)]
pub fn distribution_build(package: Package, target: Target, release_build: bool) -> crate::Result {
    let mut command = CROSS.command();

    command
        .arg("build")
        .arg("--package").arg(package.ident())
        .arg("--target-dir").arg(cross_target_dir().as_os_str()) //explicitly set target-base-dir to fix unreliable caching behavior
        .arg("--target").arg(target.to_string())
        .arg("--release");

    if release_build.not() {
        command.env("CARGO_SUPPRESS_SHADOW_REBUILD", "true"); //environment variables need to be prefixed with "CARGO_" to be passed through: https://github.com/cross-rs/cross/wiki/Configuration#environment-variable-passthrough
    }

    command.run_requiring_success()
}

pub fn out_file(package: Package, target: Target) -> PathBuf {
    cross_target_dir().join(target.to_string()).join("release").join(package.ident())
}

fn cross_target_dir() -> PathBuf {
    let cargo_target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| constants::target_dir());
    cargo_target_dir.join("cross")
}
