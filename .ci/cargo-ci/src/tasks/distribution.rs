use crate::fs;
use std::path::PathBuf;
use cicero::distribution::build::Target;
use flate2::Compression;
use tracing::debug;

use crate::{constants, Package};

/// Build and bundle a release distribution
#[derive(Debug, clap::Parser)]
#[command(alias="dist")]
pub struct DistributionCli {
    /// The operating system and CPU architecture to build for
    #[arg(long, default_value_t)]
    pub target: Target,

    /// Build artifacts in release mode, with optimizations
    #[arg(short='r', long="release")]
    pub release_build: bool,
}

#[tracing::instrument(skip_all)]
pub fn clean(package: &Package, target: Target) -> anyhow::Result<()> {
    let package_dir = out_package_dir(package, target);
    if package_dir.exists() {
        fs::remove_dir_all(&package_dir)?;
        debug!("Cleaned distribution directory at: {package_dir:?}");
    }
    Ok(())
}

#[tracing::instrument(skip_all)]
pub fn collect_executables(package: &Package, target: Target) -> anyhow::Result<()> {

    let out_dir = out_package_dir(package, target);
    fs::create_dir_all(&out_dir)?;

    fs::copy(
        crate::tasks::build::out_file(package, target),
        out_dir.join(package.name),
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
        pub target: Target,

        #[arg(long)]
        /// Skip the generation of the license files and attempt to copy them directly.
        pub skip_generate: bool,
    }
    impl DistributionCopyLicenseJsonCli {
        pub fn run(&self, package: &Package) -> anyhow::Result<()> {
            copy_license_json(package, self.target, self.skip_generate.into())
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
    pub fn copy_license_json(package: &Package, target: Target, skip_generate: SkipGenerate) -> anyhow::Result<()> {

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
    pub fn out_file(package: &Package, target: Target) -> PathBuf {
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
        target: Target,
    }
    impl DistributionBundleFilesCli {
        pub fn run(&self, package: &Package) -> anyhow::Result<()> {
            let release_build = true; //this CLI is only used in CI
            bundle_files(package, self.target, release_build)
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn bundle_files(package: &Package, target: Target, release_build: bool) -> anyhow::Result<()> {
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
            GzEncoder::new(out_file, select_compression_level(release_build))
        );
        tar_gz.append_dir_all(package.name, &in_dir)?;
        tar_gz.into_inner()?.finish()?;

        fs::remove_dir_all(in_dir)?;

        Ok(())
    }

    pub fn out_file(package: &Package, target: Target) -> PathBuf {
        let out_file_name_without_version = out_file_name_without_version(package, target);
        let version = crate::build::PKG_VERSION;

        out_arch_dir(target)
            .join(format!("{out_file_name_without_version}{version}.tar.gz"))
    }

    fn out_file_name_without_version(package: &Package, target: Target) -> String {
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
        pub target: Target,
    }
}

pub fn out_dir() -> PathBuf {
    constants::target_dir().join("distribution")
}

pub fn out_arch_dir(target: Target) -> PathBuf {
    out_dir().join(target.to_string())
}

pub fn out_package_dir(package: &Package, target: Target) -> PathBuf {
    out_arch_dir(target).join(package.name)
}

/// Choose best compression level for different tasks.
///
/// Benchmarked compression levels for EDGAR debug distribution on 2025-12-18 (resulting archive file size + tracing-timing for bundle_files):
///   best:    31.1MB, 23.7s
///   default: 31.2MB, 11.9s
///   fast:    36.0MB, 5.24s
///   none:    70.3MB, 2.86s
///
/// The file-size jump from `fast` to `default` hardly warrants differentiating for EDGAR,
/// but the hope is that the CARL distribution benefits more from this.
fn select_compression_level(release_build: bool) -> Compression {
    if release_build {
        Compression::default()
    } else {
        Compression::fast()
    }
}
