use std::process::Command;
use std::path::PathBuf;

use crate::{constants, util, Package};
use crate::core::dependency::Crate;
use crate::Target;
use crate::core::types::parsing::target::TargetSelection;
use crate::util::RunRequiringSuccess;


/// Perform a release build, without bundling a distribution.
#[derive(Debug, clap::Parser)]
pub struct BuildCli {
    #[arg(long, default_value_t)]
    pub target: TargetSelection,
}

#[tracing::instrument]
pub fn build_release(package: Package, target: Target) -> crate::Result {
    util::install_crate(Crate::Cross)?;

    Command::new("cross")
        .args([
            "build",
            "--release",
            "--all-features",
            "--package",
            &package.ident(),
            "--target-dir",
            &cross_target_dir().display().to_string(), //explicitly set target-base-dir to fix unreliable caching behavior
            "--target",
            &target.triple(),
        ])
        .run_requiring_success()?;
    Ok(())
}

pub fn out_dir(package: Package, target: Target) -> PathBuf {
    cross_target_dir().join(target.triple()).join("release").join(package.ident())
}

fn cross_target_dir() -> PathBuf {
    constants::target_dir().join("cross")
}
