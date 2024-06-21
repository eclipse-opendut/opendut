use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;
use tracing::debug;

use crate::{Arch, Package};
use crate::core::types::parsing::package::PackageSelection;
use crate::core::types::parsing::target::TargetSelection;

pub const SUPPORTED_ARCHITECTURES: [Arch; 3] = [Arch::X86_64, Arch::Armhf, Arch::Arm64];

const SELF_PACKAGE: Package = Package::Edgar;


/// Tasks available or specific for EDGAR
#[derive(clap::Parser)]
#[command(alias="opendut-edgar")]
pub struct EdgarCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    Distribution(crate::tasks::distribution::DistributionCli),
    Licenses(crate::tasks::licenses::LicensesCli),
    Run(crate::tasks::run::RunCli),

    DistributionBuild(crate::tasks::build::DistributionBuildCli),
    #[command(hide=true)]
    /// Download the NetBird Client artifact, as it normally happens when building a distribution.
    /// Intended for parallelization in CI/CD.
    DistributionNetbirdClient {
        #[arg(long, default_value_t)]
        target: TargetSelection,
    },
    #[command(hide=true)]
    DistributionRperf{
        #[arg(long, default_value_t)]
        target: TargetSelection,
    },
    DistributionCopyLicenseJson(crate::tasks::distribution::copy_license_json::DistributionCopyLicenseJsonCli),
    DistributionBundleFiles(crate::tasks::distribution::bundle::DistributionBundleFilesCli),
    DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli),
}

impl EdgarCli {
    #[tracing::instrument(name="edgar", skip(self))]
    pub fn default_handling(self) -> crate::Result {
        match self.task {
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target }) => {
                for target in target.iter() {
                    build::build_release(target)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target }) => {
                for target in target.iter() {
                    distribution::edgar_distribution(target)?;
                }
            }
            TaskCli::Licenses(cli) => cli.default_handling(PackageSelection::Single(SELF_PACKAGE))?,
            TaskCli::Run(cli) => cli.default_handling(SELF_PACKAGE)?,

            TaskCli::DistributionNetbirdClient { target } => {
                for target in target.iter() {
                    distribution::netbird::netbird_client_distribution(target)?;
                }
            }
            TaskCli::DistributionRperf { target } => {
                for target in target.iter() {
                    distribution::rperf::rperf_distribution(target)?;
                }
            }
            TaskCli::DistributionCopyLicenseJson(cli) => cli.default_handling(SELF_PACKAGE)?,
            TaskCli::DistributionBundleFiles(cli) => cli.default_handling(SELF_PACKAGE)?,
            TaskCli::DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli { target }) => {
                for target in target.iter() {
                    distribution::validate::validate_contents(target)?;
                }
            }
        };
        Ok(())
    }
}


pub mod build {
    use super::*;

    pub fn build_release(target: Arch) -> crate::Result {
        crate::tasks::build::distribution_build(SELF_PACKAGE, target)
    }
    pub fn out_dir(target: Arch) -> PathBuf {
        crate::tasks::build::out_dir(SELF_PACKAGE, target)
    }
}

pub mod distribution {
    use crate::tasks::distribution::copy_license_json::SkipGenerate;

    use super::*;

    #[tracing::instrument]
    pub fn edgar_distribution(target: Arch) -> crate::Result {
        use crate::tasks::distribution;

        let _ = netbird::map_target(target)?; //check target supported

        distribution::clean(SELF_PACKAGE, target)?;

        crate::tasks::build::distribution_build(SELF_PACKAGE, target)?;

        distribution::collect_executables(SELF_PACKAGE, target)?;

        netbird::netbird_client_distribution(target)?;
        
        rperf::rperf_distribution(target)?;
        
        distribution::copy_license_json::copy_license_json(SELF_PACKAGE, target, SkipGenerate::No)?;

        distribution::bundle::bundle_files(SELF_PACKAGE, target)?;

        validate::validate_contents(target)?;

        Ok(())
    }


    pub mod netbird {
        use anyhow::bail;
        use super::*;

        #[tracing::instrument]
        pub fn netbird_client_distribution(target: Arch) -> crate::Result {
            //Modelled after documentation here: https://docs.netbird.io/how-to/getting-started#binary-install

            let metadata = crate::metadata::cargo();
            let version = metadata.workspace_metadata["ci"]["netbird"]["version"].as_str()
                .ok_or(anyhow!("NetBird version not defined."))?;

            let os = "linux";

            let arch = map_target(target)?;

            let folder_name = format!("v{version}");
            let file_name = format!("netbird_{version}_{os}_{arch}.tar.gz");

            let netbird_artifact = download_dir().join(&folder_name).join(&file_name);
            fs::create_dir_all(netbird_artifact.parent().unwrap())?;

            if !netbird_artifact.exists() { //download
                let url = format!("https://github.com/reimarstier/netbird/releases/download/{folder_name}/{file_name}");

                debug!("Downloading netbird_{version}_{os}_{arch}.tar.gz...");
                let bytes = reqwest::blocking::get(url)?
                    .error_for_status()?
                    .bytes()?;
                debug!("Retrieved {} bytes.", bytes.len());

                fs::write(&netbird_artifact, bytes)
                    .map_err(|cause| anyhow!("Error while writing to '{}': {cause}", netbird_artifact.display()))?;
            }
            assert!(netbird_artifact.exists());

            let out_file = out_file(SELF_PACKAGE, target);
            fs::create_dir_all(out_file.parent().unwrap())?;

            fs::copy(&netbird_artifact, &out_file)
                .map_err(|cause| anyhow!("Error while copying from '{}' to '{}': {cause}", netbird_artifact.display(), out_file.display()))?;
            debug!("Placed NetBird distribution into: {}", out_file.display());

            Ok(())
        }

        pub(super) fn map_target(target: Arch) -> anyhow::Result<&'static str> {
            assert!(SUPPORTED_ARCHITECTURES.contains(&target));

            match target {
                Arch::X86_64 => Ok("amd64"),
                Arch::Arm64 => Ok("arm64"),
                Arch::Armhf => Ok("armv6"),
                other => bail!(
                    "Building a distribution for EDGAR isn't currently supported for '{}'.\n\
                    Supported targets are: {}",
                    other.triple(),
                    SUPPORTED_ARCHITECTURES.map(|arch| arch.triple()).join(", "),
                ),
            }
        }

        fn download_dir() -> PathBuf {
            crate::constants::target_dir().join("netbird")
        }

        pub fn out_file(package: Package, target: Arch) -> PathBuf {
            crate::tasks::distribution::out_package_dir(package, target).join("install").join("netbird.tar.gz")
        }
    }
    pub mod rperf {
        use std::fs::File;
        use std::path::Path;
        use std::process::Command;
        use flate2::read::GzDecoder;
        use tar::Archive;
        use crate::core::dependency::Crate;
        use crate::core::util;
        use crate::core::util::RunRequiringSuccess;
        use super::*;

        #[tracing::instrument]
        pub fn rperf_distribution(target: Arch) -> crate::Result {
            let metadata = crate::metadata::cargo();
            let version = metadata.workspace_metadata["ci"]["rperf"]["version"].as_str()
                .ok_or(anyhow!("Rperf version not defined."))?;

            let rperf_archive_folder = download_dir().join(format!("archive_{version}"));
            let rperf_archive = rperf_archive_folder.join(format!("rperf_{version}.tar.gz"));
            fs::create_dir_all(rperf_archive.parent().unwrap())?;

            if !rperf_archive.exists() {
                download_rperf_repository(version, &rperf_archive)?;
            }
            assert!(rperf_archive.exists());

            let temp_dir_path = std::env::temp_dir()
                .join("opendut-ci-edgar-rperf-distribution-b31c2679-4669-4a9c-88bd-53ebd3e06373"); //build outside the target-dir, because otherwise rperf is thought to be part of this Cargo workspace
            let temp_dir_subpath = unpack_rperf_repository(&rperf_archive, &temp_dir_path, version)?;
            
            let rperf_binary = build_rperf(&temp_dir_path, &temp_dir_subpath, target)?;

            let out_file = out_file(SELF_PACKAGE, target);
            fs::create_dir_all(out_file.parent().unwrap())?;
            fs::copy(&rperf_binary, &out_file)
                .map_err(|cause| anyhow!("Error while copying from '{}' to '{}': {cause}", rperf_binary.display(), out_file.display()))?;
            debug!("Placed rperf distribution into: {}", out_file.display());

            Ok(())
        }
        fn download_rperf_repository(version: &str, rperf_artifact: &Path) -> crate::Result {
            let url = format!("https://github.com/opensource-3d-p/rperf/archive/refs/tags/v{version}.tar.gz");

            debug!("Downloading rperf_v{version}.tar.gz...");
            let bytes = reqwest::blocking::get(url)?
                .error_for_status()?
                .bytes()?;
            debug!("Retrieved {} bytes.", bytes.len());

            fs::write(rperf_artifact, bytes)
                .map_err(|cause| anyhow!("Error while writing to '{}': {cause}", rperf_artifact.display()))?;
            Ok(())
        }
        fn unpack_rperf_repository(rperf_artifact: &Path, temp_dir_path: &Path, version: &str) -> Result<PathBuf, anyhow::Error> {
            let tar_gz = File::open(rperf_artifact)?;
            let tar = GzDecoder::new(tar_gz);
            let mut archive = Archive::new(tar);

            archive.unpack(temp_dir_path)?;
            let temp_dir_subpath = temp_dir_path.join(format!("rperf-{version}"));
            debug!("The rperf repository was unpacked to {:?}", temp_dir_subpath);
            
            Ok(temp_dir_subpath)
        }
        fn build_rperf(target_directory: &Path, current_directory: &Path, target: Arch) -> Result<PathBuf, anyhow::Error>  {
            util::install_crate(Crate::Cross)?;
            
            Command::new("cross")
                .arg("build")
                .arg("--release")
                .arg("--all-features")
                .arg("--target-dir").arg(target_directory)
                .arg("--target").arg(target.triple())
                .env("RUSTFLAGS", "-Awarnings") //ignore warnings from rperf source code
                .current_dir(current_directory)
                .run_requiring_success()?;

            let out_dir = target_directory.join(target.triple()).join("release").join("rperf");
            debug!("The rperf was built to {}", out_dir.display());
            
            Ok(out_dir)
        }
        fn download_dir() -> PathBuf {
            crate::constants::target_dir().join("rperf")
        }

        pub fn out_file(package: Package, target: Arch) -> PathBuf {
            crate::tasks::distribution::out_package_dir(package, target).join("install").join("rperf")
        }
    }

    pub mod validate {
        use std::fs::File;

        use assert_fs::prelude::*;
        use flate2::read::GzDecoder;
        use predicates::path;

        use crate::core::util::file::ChildPathExt;
        use crate::tasks::distribution::bundle;

        use super::*;

        #[tracing::instrument]
        pub fn validate_contents(target: Arch) -> crate::Result {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let archive = bundle::out_file(SELF_PACKAGE, target);
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };

            let edgar_dir = unpack_dir.child(SELF_PACKAGE.ident());
            edgar_dir.assert(path::is_dir());

            let opendut_edgar_executable = edgar_dir.child(SELF_PACKAGE.ident());
            let install_dir = edgar_dir.child("install");
            let licenses_dir = edgar_dir.child("licenses");

            edgar_dir.dir_contains_exactly_in_order(vec![
                &install_dir,
                &licenses_dir,
                &opendut_edgar_executable,
            ]);

            opendut_edgar_executable.assert_non_empty_file();
            install_dir.assert(path::is_dir());
            licenses_dir.assert(path::is_dir());

            {   //validate install dir contents
                let netbird_archive = install_dir.child("netbird.tar.gz");
                let rperf_executable = install_dir.child("rperf");

                install_dir.dir_contains_exactly_in_order(vec![
                    &netbird_archive,
                    &rperf_executable,
                ]);

                netbird_archive.assert_non_empty_file();
                rperf_executable.assert_non_empty_file();
            }

            {   //validate licenses dir contents
                let licenses_edgar_file = licenses_dir.child("opendut-edgar.licenses.json");

                licenses_dir.dir_contains_exactly_in_order(vec![
                    &licenses_edgar_file,
                ]);

                licenses_edgar_file.assert_non_empty_file();
            }

            Ok(())
        }
    }
}
