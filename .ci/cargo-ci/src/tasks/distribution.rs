use crate::fs;
use std::path::PathBuf;
use tracing::debug;

use crate::{Arch, constants, Package};
use crate::types::parsing::target::TargetSelection;

/// Build and bundle a release distribution
#[derive(Debug, clap::Parser)]
#[command(alias="dist")]
pub struct DistributionCli {
    #[arg(long, default_value_t)]
    pub target: TargetSelection,
}

/// Build and bundle a release distribution
#[derive(Debug, clap::Parser)]
#[command(alias="dist")]
pub struct DistributionCliWithFilter {
    #[arg(long, default_value_t)]
    pub target: TargetSelection,
    /// Build only certain sub-paths of the distribution, specified with Glob patterns (relative to the distribution root).
    #[arg(long)]
    pub filter: Vec<cicero::distribution::filter::Pattern>,
}

#[tracing::instrument(skip_all)]
pub fn clean(package: Package, target: Arch) -> crate::Result {
    let package_dir = out_package_dir(package, target);
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)?;
        debug!("Cleaned distribution directory at: {package_dir:?}");
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn collect_executables(package: Package, target: Arch) -> crate::Result {

    let out_dir = out_package_dir(package, target);
    fs::create_dir_all(&out_dir)?;

    fs::copy(
        crate::tasks::build::out_file(package, target),
        out_dir.join(package.ident()),
    )?;
    Ok(())
}


pub mod copy_license_json {
    use tracing::info;
    use super::*;

    /// Copy license files to the distribution directory, as it normally happens when building a distribution.
    /// Intended for parallelization in CI/CD.
    #[derive(Debug, clap::Parser)]
    #[command(hide=true)]
    pub struct DistributionCopyLicenseJsonCli {
        #[arg(long, default_value_t)]
        pub target: TargetSelection,

        #[arg(long)]
        /// Skip the generation of the license files and attempt to copy them directly.
        pub skip_generate: bool,
    }
    impl DistributionCopyLicenseJsonCli {
        pub fn default_handling(&self, package: Package) -> crate::Result {
            for target in self.target.iter() {
                copy_license_json(package, target, self.skip_generate.into())?;
            }
            Ok(())
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub enum SkipGenerate { Yes, No }
    impl From<bool> for SkipGenerate {
        fn from(value: bool) -> Self {
            if value { SkipGenerate::Yes } else { SkipGenerate::No }
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn copy_license_json(package: Package, target: Arch, skip_generate: SkipGenerate) -> crate::Result {

        match skip_generate {
            SkipGenerate::Yes => info!("Skipping generation of licenses, as requested. Directly attempting to copy to target location."),
            SkipGenerate::No => crate::tasks::licenses::json::export_json(package)?,
        };
        let licenses_file = crate::tasks::licenses::json::out_file(package);

        let out_file = out_file(package, target);
        fs::create_dir_all(out_file.parent().unwrap())?;

        fs::copy(licenses_file, out_file)?;

        Ok(())
    }
    pub fn out_file(package: Package, target: Arch) -> PathBuf {
        out_package_dir(package, target)
            .join("licenses")
            .join(crate::tasks::licenses::json::out_file_name(package))
    }
}

pub mod bundle {
    use super::*;

    /// Directly bundle files from the distribution directory, as it normally happens when building a distribution.
    /// Intended for parallelization in CI/CD.
    #[derive(Debug, clap::Parser)]
    #[command(hide=true)]
    pub struct DistributionBundleFilesCli {
        #[arg(long, default_value_t)]
        target: TargetSelection,
    }
    impl DistributionBundleFilesCli {
        pub fn default_handling(&self, package: Package) -> crate::Result {
            for target in self.target.iter() {
                bundle_files(package, target)?;
            }
            Ok(())
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn bundle_files(package: Package, target: Arch) -> crate::Result {
        use flate2::Compression;
        use flate2::write::GzEncoder;

        let in_dir = out_package_dir(package, target);

        let out_file = out_file(package, target);
        let out_parent_dir = out_file.parent().unwrap();
        fs::create_dir_all(out_parent_dir)?;

        { //delete previous distribution files
            let file_name_prefix = out_file_name_without_version(package, target);

            let files = std::fs::read_dir(out_parent_dir)?
                .map(|entry| entry.unwrap())
                .filter(|entry| entry.path().is_file())
                .filter(|entry|
                    entry.path().file_name().unwrap()
                        .to_str().unwrap()
                        .starts_with(&file_name_prefix)
                );

            for file in files {
                fs::remove_file(file.path())?;
            }
        }

        let out_file = fs::File::create(out_file)?;

        let mut tar_gz = tar::Builder::new(
            //TODO Optimize the way the CARL distribution is built (don't remove all files every time), then switch this back to Compression::best().
            // While benchmarking the EDGAR distribution, Compression::best() took 20+ seconds for 19MB. Compression::fast() took 7 seconds for 21MB.
            GzEncoder::new(out_file, Compression::fast())
        );
        tar_gz.append_dir_all(package.ident(), &in_dir)?;
        tar_gz.into_inner()?.finish()?;

        fs::remove_dir_all(in_dir)?;

        Ok(())
    }

    pub fn out_file(package: Package, target: Arch) -> PathBuf {
        let out_file_name_without_version = out_file_name_without_version(package, target);
        let version = crate::build::PKG_VERSION;

        out_arch_dir(target)
            .join(format!("{out_file_name_without_version}{version}.tar.gz"))
    }

    fn out_file_name_without_version(package: Package, target: Arch) -> String {
        let package = package.ident();
        let target = target.triple();
        format!("{package}-{target}-")
    }
}

pub mod validate {
    use super::*;

    /// Unpack and verify the contents of the distribution, as it normally happens when building a distribution.
    /// Intended for parallelization in CI/CD.
    #[derive(Debug, clap::Parser)]
    #[command(hide=true)]
    pub struct DistributionValidateContentsCli {
        #[arg(long, default_value_t)]
        pub target: TargetSelection,
    }
}

pub fn out_dir() -> PathBuf {
    constants::target_dir().join("distribution")
}

pub fn out_arch_dir(target: Arch) -> PathBuf {
    out_dir().join(target.triple())
}

pub fn out_package_dir(package: Package, target: Arch) -> PathBuf {
    out_arch_dir(target).join(package.ident())
}
