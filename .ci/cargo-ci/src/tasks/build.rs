use std::ops::Not;
use crate::fs;
use std::path::{Path, PathBuf};

use cicero::command_exit_ok::CommandExitOk;
use cicero::distribution::build::Target;

use crate::{constants, Package};
use crate::core::commands::CROSS;


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
pub fn distribution_build(package: &Package, target: Target, release_build: bool) -> anyhow::Result<()> {
    let mut command = CROSS.command();

    command
        .arg("build")
        .arg("--package").arg(package.name)
        .arg("--target-dir").arg(cross_target_dir().as_os_str()) //explicitly set target-base-dir to fix unreliable caching behavior
        .arg("--target").arg(target.to_string())
        .arg("--release");

    if release_build.not() {
        command.env("CARGO_SUPPRESS_SHADOW_REBUILD", "true"); //environment variables need to be prefixed with "CARGO_" to be passed through: https://github.com/cross-rs/cross/wiki/Configuration#environment-variable-passthrough
    }

    command.status_exit_ok()?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn distribution_build_with_out_path(package: &Package, target: Target, out: &Path, release_build: bool) -> anyhow::Result<()> {
    distribution_build(package, target, release_build)?;

    let source_file = out_file(package, target);

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::rename(source_file, out)?;
    Ok(())
}

pub fn out_file(package: &Package, target: Target) -> PathBuf {
    cross_target_dir().join(target.to_string()).join("release").join(package.name)
}

fn cross_target_dir() -> PathBuf {
    let cargo_target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| constants::target_dir());
    cargo_target_dir.join("cross")
}
