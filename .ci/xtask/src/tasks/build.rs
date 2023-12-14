use std::process::Command;
use std::path::PathBuf;

use crate::{constants, util, Package};
use crate::Arch;
use crate::util::RunRequiringSuccess;


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
