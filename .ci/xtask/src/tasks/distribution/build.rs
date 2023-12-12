use std::process::Command;
use std::path::PathBuf;

use crate::{constants, util, Package};
use crate::Arch;


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
            &target_dir().display().to_string(), //explicitly set target-base-dir to fix unreliable caching behavior
            "--target",
            &target.triple(),
        ])
        .status()?;

    Ok(())
}

pub(super) fn target_dir() -> PathBuf {
    constants::target_dir().join("cross")
}
