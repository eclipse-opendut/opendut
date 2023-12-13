use std::fs;
use std::path::PathBuf;

use crate::{constants, Package, Arch};

pub mod build;


#[tracing::instrument]
pub fn clean() -> anyhow::Result<()> {
    if distribution_dir().exists() {
        fs::remove_dir_all(distribution_dir())?;
        log::debug!("Cleaned distribution directory.");
    }
    Ok(())
}

#[tracing::instrument]
pub fn collect_executables(build_dir: PathBuf, package: &Package, target: &Arch) -> anyhow::Result<()> {

    let out_dir = package_dir(package, target);
    fs::create_dir_all(&out_dir)?;

    fs::copy(
        build_dir.join(&target.triple()).join("release").join(&package.ident()),
        out_dir.join(package.ident()),
    )?;
    Ok(())
}

#[tracing::instrument]
pub fn bundle_collected_files(package: &Package, target: &Arch) -> anyhow::Result<()> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let in_dir = package_dir(package, target);

    let out_dir = arch_dir(target);
    fs::create_dir_all(&out_dir)?;

    let platform = "linux";
    let arch = target.triple();
    let version = crate::build::PKG_VERSION;

    let file = fs::File::create(out_dir.join(format!("{}-{platform}-{arch}-{version}.tar.gz", package.ident())))?;
    let mut tar_gz = tar::Builder::new(
        GzEncoder::new(file, Compression::best())
    );
    tar_gz.append_dir_all(package.ident(), &in_dir)?;
    tar_gz.finish()?;
    fs::remove_dir_all(in_dir)?;

    Ok(())
}

pub fn distribution_dir() -> PathBuf {
    constants::target_dir().join("distribution")
}

pub fn arch_dir(target: &Arch) -> PathBuf {
    distribution_dir().join(target.triple())
}

pub fn package_dir(package: &Package, target: &Arch) -> PathBuf {
    arch_dir(target).join(package.ident())
}
