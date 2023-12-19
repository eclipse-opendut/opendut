use std::fs;
use std::path::PathBuf;

use crate::{constants, Package, Target};
use crate::types::parsing::target::TargetSelection;


/// Build and bundle a release distribution
#[derive(Debug, clap::Parser)]
#[command(alias="dist")]
pub struct DistributionCli {
    #[arg(long, default_value_t)]
    pub target: TargetSelection,
}

#[tracing::instrument]
pub fn clean(package: &Package, target: &Target) -> anyhow::Result<()> {
    let package_dir = out_package_dir(package, target);
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)?;
        log::debug!("Cleaned distribution directory at: {package_dir:?}");
    }
    Ok(())
}

#[tracing::instrument]
pub fn collect_executables(package: &Package, target: &Target) -> anyhow::Result<()> {

    let out_dir = out_package_dir(package, target);
    fs::create_dir_all(&out_dir)?;

    fs::copy(
        crate::tasks::build::out_dir(package, target),
        out_dir.join(package.ident()),
    )?;
    Ok(())
}

#[tracing::instrument]
pub fn bundle_collected_files(package: &Package, target: &Target) -> anyhow::Result<()> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let in_dir = out_package_dir(package, target);

    let out_dir = out_arch_dir(target);
    fs::create_dir_all(&out_dir)?;

    let target_triple = target.triple();
    let version = crate::build::PKG_VERSION;

    let file = fs::File::create(
        out_dir.join(format!("{}-{target_triple}-{version}.tar.gz", package.ident()))
    )?;

    let mut tar_gz = tar::Builder::new(
        GzEncoder::new(file, Compression::best())
    );
    tar_gz.append_dir_all(package.ident(), &in_dir)?;
    tar_gz.finish()?;

    fs::remove_dir_all(in_dir)?;

    Ok(())
}

pub mod licenses {
    use super::*;

    #[tracing::instrument]
    pub fn get_licenses(package: &Package, target: &Target) -> anyhow::Result<()> {

        crate::tasks::licenses::json::export_json(package)?;
        let licenses_file = crate::tasks::licenses::json::out_file(package);

        let out_dir = out_file(package, target);
        fs::create_dir_all(out_dir.parent().unwrap())?;

        fs::copy(licenses_file, out_dir)?;

        Ok(())
    }
    pub fn out_file(package: &Package, target: &Target) -> PathBuf {
        let licenses_file_name = format!("{}.licenses.json", package.ident());
        out_package_dir(package, target).join("licenses").join(licenses_file_name)
    }
}

pub fn out_dir() -> PathBuf {
    constants::target_dir().join("distribution")
}

pub fn out_arch_dir(target: &Target) -> PathBuf {
    out_dir().join(target.triple())
}

pub fn out_package_dir(package: &Package, target: &Target) -> PathBuf {
    out_arch_dir(target).join(package.ident())
}
