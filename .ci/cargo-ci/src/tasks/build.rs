use crate::fs;
use std::path::{Path, PathBuf};

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
}

#[tracing::instrument(skip_all)]
pub fn distribution_build(package: Package, target: Arch) -> crate::Result {
    CROSS.command()
        .arg("--package").arg(package.ident())
        .arg("--target-dir").arg(cross_target_dir().as_os_str()) //explicitly set target-base-dir to fix unreliable caching behavior
        .arg("--target").arg(target.triple())
        .run_requiring_success()?;
    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn distribution_build_with_out_path(package: Package, target: Arch, out: &Path) -> crate::Result {
    distribution_build(package, target)?;

    let source_file = out_file(package, target);
    fs::create_dir_all(out.parent().unwrap())?;

    fs::rename(
        source_file,
        out
    )?;

    Ok(())
}

pub fn out_file(package: Package, target: Arch) -> PathBuf {
    cross_target_dir().join(target.triple()).join("release").join(package.ident())
}

fn cross_target_dir() -> PathBuf {
    constants::target_dir().join("cross")
}
