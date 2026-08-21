use crate::{fs, workspace};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context};
use cicero::distribution::build::{target, Target};
use cicero::command_exit_ok::CommandExitOk;
use tracing::debug;

use crate::Package;
use crate::core::types::parsing::package::PackageSelection;

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
    Distribution(crate::tasks::distribution::DistributionCliWithFilter),
    Licenses(crate::tasks::licenses::LicensesCli),
    Run(crate::tasks::run::RunCli),
    Docker(crate::tasks::docker::DockerCli),
}

impl EdgarCli {
    #[tracing::instrument(name="edgar", skip_all)]
    pub fn run(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::Distribution(crate::tasks::distribution::DistributionCliWithFilter { target, release_build, filter, output_dir }) => {
                let filter = if filter.is_empty() {
                    cicero::distribution::filter::DistributionFilter::Disabled
                } else {
                    cicero::distribution::filter::DistributionFilter::Enabled(filter)
                };

                let out_file = match output_dir {
                    Some(output_dir) => output_dir.join(crate::tasks::distribution::bundle::out_file_name(SELF_PACKAGE, target)),
                    None => crate::tasks::distribution::bundle::out_file(SELF_PACKAGE, target),
                };

                distribution::edgar_distribution(target, &out_file, release_build, filter)?;
            }
            TaskCli::Licenses(cli) => cli.run(PackageSelection::Single(SELF_PACKAGE.clone()))?,
            TaskCli::Run(cli) => cli.run(SELF_PACKAGE)?,

            TaskCli::Docker(implementation) => {
                implementation.run(SELF_PACKAGE)?;
            }
        };
        Ok(())
    }
}


pub mod build {
    use super::*;

    pub fn build_release(target: Target, release_build: bool) -> anyhow::Result<()> {
        crate::tasks::build::distribution_build(SELF_PACKAGE, target, release_build)
    }
}

pub mod distribution {
    use cicero::distribution::{Distribution, DistributionOptions, bundle::tar::TarBundler, filter::DistributionFilter};

    use super::*;

    #[tracing::instrument]
    pub fn edgar_distribution(target: Target, out_file: &Path, release_build: bool, filter: DistributionFilter) -> anyhow::Result<()> {
        let _ = netbird::map_target(target)?; //check target supported

        let distribution = Distribution::new_with_options(
            "opendut-edgar",
            DistributionOptions { filter: filter.clone() },
        )?;

        distribution.add_file("opendut-edgar", |out_file| {
            crate::tasks::build::distribution_build_with_out_path(SELF_PACKAGE, target, out_file, release_build)
        })?;

        distribution
            .dir("install")?
            .add_file("netbird.tar.gz", |file| netbird::netbird_client_distribution(target, file))?
            .add_file("rperf", |file| rperf::rperf_distribution(target, file))?;

        distribution
            .dir("plugins")?
            .add_file("plugins.txt", |file| {
                fs::File::create(file)
                    .context("Error when creating empty plugins.txt.")?;
                Ok(())
            })?;

        distribution.dir("licenses")?
            .add_file("opendut-edgar.licenses.json", |out_file| crate::tasks::licenses::json::export_json_with_out_path(SELF_PACKAGE, out_file))?;


        if let DistributionFilter::Disabled = filter {
            let distribution_path = distribution.bundle(TarBundler::default())?;
            validate::validate_contents_of(&distribution_path)?;

            if let Some(parent_dir) = out_file.parent() {
                fs::create_dir_all(parent_dir)?;
            }
            fs::rename(distribution_path, out_file)?;
        };

        Ok(())
    }


    pub mod netbird {
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn netbird_client_distribution(target: Target, out_file: &Path) -> anyhow::Result<()> {
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

            fs::create_dir_all(out_file.parent().unwrap())?;

            fs::copy(&netbird_artifact, out_file)
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
    }

    pub mod rperf {
        use crate::fs::File;
        use std::path::Path;
        use flate2::read::GzDecoder;
        use tar::Archive;
        use crate::core::commands::CROSS;
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn rperf_distribution(target: Target, out_file: &Path) -> anyhow::Result<()> {
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
            assert!(&rperf_binary.exists());

            fs::create_dir_all(out_file.parent().unwrap())?;
            fs::copy(&rperf_binary, out_file)
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
        use assert_fs::prelude::*;
        use flate2::read::GzDecoder;
        use predicates::path;

        use crate::core::util::file::ChildPathExt;

        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn validate_contents_of(path: &Path) -> anyhow::Result<()> {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let mut archive = tar::Archive::new(GzDecoder::new(fs::File::open(path)?));
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
