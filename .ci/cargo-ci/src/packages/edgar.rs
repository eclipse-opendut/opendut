use crate::{fs, workspace};
use std::path::PathBuf;

use anyhow::{anyhow, bail, Context};
use cicero::distribution::build::{target, Target};
use cicero::command_exit_ok::CommandExitOk;
use tracing::debug;

use crate::Package;
use crate::core::types::parsing::package::PackageSelection;
use crate::tasks::build::BuildArgs;

pub const SUPPORTED_TARGETS: [Target; 3] = [target::x86_64_unknown_linux_gnu, target::armv7_unknown_linux_gnueabihf, target::aarch64_unknown_linux_gnu];

const SELF_PACKAGE: &Package = &workspace::package::opendut_edgar;


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
        target: Target,
    },
    #[command(hide=true)]
    DistributionRperf {
        #[arg(long, default_value_t)]
        target: Target,
    },
    #[command(hide=true)]
    DistributionPluginsDir {
        #[arg(long, default_value_t)]
        target: Target,
    },
    DistributionCopyLicenseJson(crate::tasks::distribution::copy_license_json::DistributionCopyLicenseJsonCli),
    DistributionBundleFiles(crate::tasks::distribution::bundle::DistributionBundleFilesCli),
    DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli),
    Docker(crate::tasks::docker::DockerCli),
}

impl EdgarCli {
    #[tracing::instrument(name="edgar", skip_all)]
    pub fn run(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target, build_args }) => {
                build::build_release(target, &build_args)?;
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target, build_args }) => {
                distribution::edgar_distribution(target, &build_args)?;
            }
            TaskCli::Licenses(cli) => cli.run(PackageSelection::Single(SELF_PACKAGE.clone()))?,
            TaskCli::Run(cli) => cli.run(SELF_PACKAGE)?,

            TaskCli::DistributionNetbirdClient { target } => {
                distribution::netbird::netbird_client_distribution(target)?;
            }
            TaskCli::DistributionRperf { target } => {
                distribution::rperf::rperf_distribution(target)?;
            }
            TaskCli::DistributionPluginsDir { target } => {
                distribution::plugins::empty_plugins_dir(target)?
            }
            TaskCli::DistributionCopyLicenseJson(cli) => cli.run(SELF_PACKAGE)?,
            TaskCli::DistributionBundleFiles(cli) => cli.run(SELF_PACKAGE)?,
            TaskCli::DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli { target }) => {
                distribution::validate::validate_contents(target)?;
            }
            TaskCli::Docker(implementation) => {
                implementation.run(SELF_PACKAGE)?;
            }
        };
        Ok(())
    }
}


pub mod build {
    use super::*;

    pub fn build_release(target: Target, build_args: &BuildArgs) -> anyhow::Result<()> {
        crate::tasks::build::distribution_build(SELF_PACKAGE, target, build_args)
    }
}

pub mod distribution {
    use crate::tasks::distribution::copy_license_json::SkipGenerate;

    use super::*;

    #[tracing::instrument]
    pub fn edgar_distribution(target: Target, build_args: &BuildArgs) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        let _ = netbird::map_target(target)?; //check target supported

        crate::tasks::build::distribution_build(SELF_PACKAGE, target, build_args)?;

        cicero::cache::Output::from(
            distribution::bundle::out_file(SELF_PACKAGE, target)
        ).rebuild_on_change(
            [crate::tasks::build::out_file(SELF_PACKAGE, target)],
            || {

                distribution::clean(SELF_PACKAGE, target)?;

                distribution::collect_executables(SELF_PACKAGE, target)?;

                netbird::netbird_client_distribution(target)?; //TODO rebuild cache when this changes (we currently accept the risk of this being false, since the NetBird code does not change often)

                rperf::rperf_distribution(target)?; //TODO rebuild cache when this changes (we currently accept the risk of this being false, since the Rperf code does not change often)

                plugins::empty_plugins_dir(target)?;

                distribution::copy_license_json::copy_license_json(SELF_PACKAGE, target, SkipGenerate::No)?;

                distribution::bundle::bundle_files(SELF_PACKAGE, target, build_args.release_build)?;

                validate::validate_contents(target)?;

                Ok(())
            })?;

        Ok(())
    }


    pub mod netbird {
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn netbird_client_distribution(target: Target) -> anyhow::Result<()> {
            //Modelled after documentation here: https://docs.netbird.io/how-to/getting-started#binary-install

            let metadata = crate::metadata::cargo();
            let version = metadata.workspace_metadata["ci"]["netbird"]["version"].as_str()
                .ok_or(anyhow!("NetBird version not defined."))?;
            let repository = metadata.workspace_metadata["ci"]["netbird"]["repository"].as_str()
                .ok_or(anyhow!("NetBird repository not defined."))?;

            let os = "linux";

            let arch = map_target(target)?;

            let folder_name = format!("v{version}");
            let file_name = format!("netbird_{version}_{os}_{arch}.tar.gz");

            let netbird_artifact = download_dir().join(&folder_name).join(&file_name);
            fs::create_dir_all(netbird_artifact.parent().unwrap())?;

            if !netbird_artifact.exists() { //download
                let url = format!("{repository}/releases/download/{folder_name}/{file_name}");

                debug!("Downloading netbird_{version}_{os}_{arch}.tar.gz...");
                let bytes = reqwest::blocking::get(url)?
                    .error_for_status()?
                    .bytes()?;
                debug!("Retrieved {} bytes.", bytes.len());

                fs::write(&netbird_artifact, bytes)
                    .context(format!("Error while writing to {netbird_artifact:?}"))?;
            }
            assert!(netbird_artifact.exists());

            let out_file = out_file(SELF_PACKAGE, target);
            fs::create_dir_all(out_file.parent().unwrap())?;

            fs::copy(&netbird_artifact, &out_file)
                .context(format!("Error while copying from {netbird_artifact:?} to {out_file:?}"))?;
            debug!("Placed NetBird distribution into: {out_file:?}");

            Ok(())
        }

        pub(super) fn map_target(target: Target) -> anyhow::Result<&'static str> {
            assert!(SUPPORTED_TARGETS.contains(&target));

            match target {
                target::x86_64_unknown_linux_gnu => Ok("amd64"),
                target::aarch64_unknown_linux_gnu => Ok("arm64"),
                target::armv7_unknown_linux_gnueabihf => Ok("armv6"),
                other => bail!(
                    "Building a distribution for EDGAR isn't currently supported for '{other}'.\n\
                    Supported targets are: {}",
                    SUPPORTED_TARGETS.map(|target| target.to_string()).join(", "),
                ),
            }
        }

        fn download_dir() -> PathBuf {
            crate::constants::target_dir().join("netbird")
        }

        pub fn out_file(package: &Package, target: Target) -> PathBuf {
            crate::tasks::distribution::out_package_dir(package, target).join("install").join("netbird.tar.gz")
        }
    }
    pub mod rperf {
        use crate::fs::File;
        use std::path::Path;
        use flate2::read::GzDecoder;
        use tar::Archive;
        use crate::core::commands::CROSS;
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn rperf_distribution(target: Target) -> anyhow::Result<()> {
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

            assert!(&rperf_binary.exists());

            fs::create_dir_all(out_file.parent().unwrap())?;
            fs::copy(&rperf_binary, &out_file)
                .context(format!("Error while copying from {rperf_binary:?} to {out_file:?}"))?;
            debug!("Placed rperf distribution into: {out_file:?}");

            Ok(())
        }
        fn download_rperf_repository(version: &str, rperf_artifact: &Path) -> anyhow::Result<()> {
            let url = format!("https://github.com/opensource-3d-p/rperf/archive/refs/tags/v{version}.tar.gz");

            debug!("Downloading rperf_v{version}.tar.gz...");
            let bytes = reqwest::blocking::get(url)?
                .error_for_status()?
                .bytes()?;
            debug!("Retrieved {} bytes.", bytes.len());

            fs::write(rperf_artifact, bytes)
                .context(format!("Error while writing to {rperf_artifact:?}"))?;
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
        fn build_rperf(target_directory: &Path, current_directory: &Path, target: Target) -> Result<PathBuf, anyhow::Error>  {
            CROSS.command()
                .arg("build")
                .arg("--release")
                .arg("--all-features")
                .arg("--target-dir").arg(target_directory)
                .arg("--target").arg(target.to_string())
                .env("RUSTFLAGS", "-Awarnings") //ignore warnings from rperf source code
                .current_dir(current_directory)
                .status_exit_ok()?;

            let out_dir = target_directory.join(target.to_string()).join("release").join("rperf");
            debug!("The rperf distribution was built to {out_dir:?}");

            Ok(out_dir)
        }
        fn download_dir() -> PathBuf {
            crate::constants::target_dir().join("rperf")
        }

        pub fn out_file(package: &Package, target: Target) -> PathBuf {
            crate::tasks::distribution::out_package_dir(package, target).join("install").join("rperf")
        }
    }

    pub mod plugins {
        use fs_err::File;
        use crate::tasks::distribution::out_package_dir;
        use super::*;

        pub fn empty_plugins_dir(target: Target) -> anyhow::Result<()> {
            let plugins_dir = out_package_dir(SELF_PACKAGE, target).join("plugins");
            fs::create_dir_all(&plugins_dir)?;

            let plugins_file = plugins_dir.join("plugins.txt");
            File::create(plugins_file)?;

            Ok(())
        }
    }

    pub mod validate {
        use crate::fs::File;

        use assert_fs::prelude::*;
        use flate2::read::GzDecoder;
        use predicates::path;

        use crate::core::util::file::ChildPathExt;
        use crate::tasks::distribution::bundle;

        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn validate_contents(target: Target) -> anyhow::Result<()> {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let archive = bundle::out_file(SELF_PACKAGE, target);
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };

            let edgar_dir = unpack_dir.child(SELF_PACKAGE.name);
            edgar_dir.assert(path::is_dir());

            let opendut_edgar_executable = edgar_dir.child(SELF_PACKAGE.name);
            let install_dir = edgar_dir.child("install");
            let licenses_dir = edgar_dir.child("licenses");
            let plugins_dir = edgar_dir.child("plugins");

            edgar_dir.dir_contains_exactly_in_order(vec![
                &install_dir,
                &licenses_dir,
                &opendut_edgar_executable,
                &plugins_dir,
            ]);

            opendut_edgar_executable.assert_non_empty_file();
            install_dir.assert(path::is_dir());
            licenses_dir.assert(path::is_dir());
            plugins_dir.assert(path::is_dir());

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

            {   //validate plugins dir contents
                let plugins_file = plugins_dir.child("plugins.txt");

                plugins_dir.dir_contains_exactly_in_order(vec![
                    &plugins_file,
                ]);

                plugins_file.assert(path::is_file());
            }

            Ok(())
        }
    }
}
