use std::process::Command;
use std::path::PathBuf;

use crate::{constants, util, Package};
use crate::Arch;
use crate::core::types::parsing::arch::ArchSelection;
use crate::util::RunRequiringSuccess;


/// Perform a release build, without bundling a distribution.
#[derive(Debug, clap::Parser)]
pub struct BuildCli {
    #[arg(long, default_value_t)]
    pub target: ArchSelection,
}

#[tracing::instrument]
pub fn build_release(package: &Package, target: &Arch) -> anyhow::Result<()> {
    util::install_crate("cross")?;

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
        .run_requiring_success();
    Ok(())
}

pub fn out_dir(package: &Package, target: &Arch) -> PathBuf {
    cross_target_dir().join(target.triple()).join("release").join(package.ident())
}

fn cross_target_dir() -> PathBuf {
    constants::target_dir().join("cross")
}
