use std::fs;
use std::path::PathBuf;

use crate::{constants, Package, Arch};

mod build;
mod opendut_carl;
mod opendut_edgar;


impl Package {
    pub fn has_distribution(&self) -> bool {
        match self {
            Package::OpendutCarl => true,
            Package::OpendutCarlApi => false,
            Package::OpendutCleo => true,
            Package::OpendutEdgar => true,
            Package::OpendutLea => false,
            Package::OpendutNetbirdClientApi => false,
            Package::OpendutTypes => false,
            Package::OpendutUtil => false,
            Package::OpendutVpn => false,
            Package::OpendutVpnNetbird => false,
            Package::OpendutIntegrationTests => false,
        }
    }
}


#[tracing::instrument]
pub fn distribution(package: &Package, target: &Arch) -> anyhow::Result<()> {

    if package.has_distribution() {
        clean()?;

        build::build_release(package, target)?;

        collect_executables(package, target)?;

        collect_package_specific_files(package, target)?;

        bundle_collected_files(package, target)?;
    }
    else {
        println!("Skipping for library crate: {package}");
    }

    Ok(())
}

#[tracing::instrument]
fn clean() -> anyhow::Result<()> {
    if distribution_dir().exists() {
        fs::remove_dir_all(distribution_dir())?;
        log::debug!("Cleaned distribution directory.");
    }
    Ok(())
}

#[tracing::instrument]
fn collect_executables(package: &Package, target: &Arch) -> anyhow::Result<()> {

    let out_dir = package_dir(target, package);
    fs::create_dir_all(&out_dir)?;

    fs::copy(
        build::target_dir().join(&target.triple()).join("release").join(&package.ident()),
        out_dir.join(package.ident()),
    )?;
    Ok(())
}

#[tracing::instrument]
fn collect_package_specific_files(package: &Package, target: &Arch) -> anyhow::Result<()> {
    match package {
        Package::OpendutEdgar => opendut_edgar::collect_edgar_specific_files(package, target)?,
        Package::OpendutCarl => opendut_carl::collect_carl_specific_files()?,
        _ => {}
    };
    Ok(())
}

#[tracing::instrument]
fn bundle_collected_files(package: &Package, target: &Arch) -> anyhow::Result<()> {
    use flate2::Compression;
    use flate2::write::GzEncoder;

    let in_dir = package_dir(target, package);

    let out_dir = arch_dir(target);
    fs::create_dir_all(&out_dir)?;

    let platform = "linux";
    let arch = target.name();
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

fn arch_dir(target: &Arch) -> PathBuf {
    distribution_dir().join(target.triple())
}

fn package_dir(target: &Arch, package: &Package) -> PathBuf {
    arch_dir(target).join(package.ident())
}
