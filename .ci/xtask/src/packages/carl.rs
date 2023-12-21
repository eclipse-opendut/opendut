use std::fs;
use std::path::PathBuf;

use crate::{Package, Target};
use crate::core::types::parsing::package::PackageSelection;
use crate::packages::carl::distribution::copy_license_json::copy_license_json;
use crate::tasks::distribution::copy_license_json::SkipGenerate;

const PACKAGE: Package = Package::Carl;

/// Tasks available or specific for CARL
#[derive(Debug, clap::Parser)]
#[command(alias="opendut-carl")]
pub struct CarlCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    Build(crate::tasks::build::BuildCli),
    Distribution(crate::tasks::distribution::DistributionCli),
    Licenses(crate::tasks::licenses::LicensesCli),

    #[command(hide=true)]
    DistributionCopyLicenseJson(crate::tasks::distribution::copy_license_json::DistributionCopyLicenseJsonCli),
    #[command(hide=true)]
    DistributionBundleFiles(crate::tasks::distribution::bundle::DistributionBundleFilesCli),
    #[command(hide=true)]
    DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli),
}

impl CarlCli {
    pub fn default_handling(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::Build(crate::tasks::build::BuildCli { target }) => {
                for target in target.iter() {
                    build::build_release(target)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target }) => {
                for target in target.iter() {
                    distribution::carl_distribution(target)?;
                }
            }
            TaskCli::Licenses(implementation) => {
                implementation.default_handling(PackageSelection::Single(PACKAGE))?;
            }

            TaskCli::DistributionCopyLicenseJson(implementation) => {
                let skip_generate = SkipGenerate::from(implementation.skip_generate);
                for target in implementation.target.iter() {
                    copy_license_json(target, skip_generate)?;
                }
            }
            TaskCli::DistributionBundleFiles(implementation) => {
                implementation.default_handling(PACKAGE)?;
            }
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

    pub fn build_release(target: Target) -> anyhow::Result<()> {
        crate::tasks::build::build_release(PACKAGE, target)
    }
    pub fn out_dir(target: Target) -> PathBuf {
        crate::tasks::build::out_dir(PACKAGE, target)
    }
}

pub mod distribution {
    use crate::tasks::distribution::copy_license_json::SkipGenerate;

    use super::*;

    #[tracing::instrument]
    pub fn carl_distribution(target: Target) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        let distribution_out_dir = distribution::out_package_dir(PACKAGE, target);

        distribution::clean(PACKAGE, target)?;

        crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(PACKAGE, target)?;

        lea::get_lea(&distribution_out_dir)?;
        copy_license_json::copy_license_json(target, SkipGenerate::No)?;

        distribution::bundle::bundle_files(PACKAGE, target)?;

        validate::validate_contents(target)?;

        Ok(())
    }

    mod lea {
        use super::*;

        #[tracing::instrument]
        pub fn get_lea(out_dir: &PathBuf) -> anyhow::Result<()> {

            crate::packages::lea::build::build_release()?;
            let lea_build_dir = crate::packages::lea::build::out_dir();

            let lea_out_dir = out_dir.join("lea");

            fs::create_dir_all(&lea_out_dir)?;

            fs_extra::dir::copy(
                lea_build_dir,
                &lea_out_dir,
                &fs_extra::dir::CopyOptions::default()
                    .overwrite(true)
                    .content_only(true)
            )?;

            Ok(())
        }
    }

    pub mod copy_license_json {
        use serde_json::json;

        use crate::tasks::distribution::copy_license_json::SkipGenerate;

        use super::*;

        #[tracing::instrument]
        pub fn copy_license_json(target: Target, skip_generate: SkipGenerate) -> anyhow::Result<()> {

            match skip_generate {
                SkipGenerate::Yes => log::info!("Skipping generation of licenses, as requested. Directly attempting to copy to target location."),
                SkipGenerate::No => {
                    for package in [PACKAGE, Package::Lea, Package::Edgar] {
                        crate::tasks::licenses::json::export_json(package)?;
                    }
                }
            };

            let carl_in_file = crate::tasks::licenses::json::out_file(PACKAGE);
            let carl_out_file = crate::tasks::distribution::copy_license_json::out_file(PACKAGE, target);
            let out_dir = carl_out_file.parent().unwrap();

            let lea_in_file = crate::tasks::licenses::json::out_file(Package::Lea);
            let lea_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(Package::Lea));
            let edgar_in_file = crate::tasks::licenses::json::out_file(Package::Edgar);
            let edgar_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(Package::Edgar));

            fs::create_dir_all(out_dir)?;
            fs::copy(carl_in_file, &carl_out_file)?;
            fs::copy(lea_in_file, &lea_out_file)?;
            fs::copy(edgar_in_file, &edgar_out_file)?;

            fs::write(
                out_dir.join("index.json"),
                json!({
                    "carl": carl_out_file.file_name().unwrap().to_str(),
                    "edgar": edgar_out_file.file_name().unwrap().to_str(),
                    "lea": lea_out_file.file_name().unwrap().to_str(),
                }).to_string(),
            )?;

            Ok(())
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
        pub fn validate_contents(target: Target) -> anyhow::Result<()> {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let archive = bundle::out_file(PACKAGE, target);
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };


            let carl_dir = unpack_dir.child("opendut-carl");
            carl_dir.assert(path::is_dir());

            let opendut_carl_executable = carl_dir.child("opendut-carl");
            let lea_dir = carl_dir.child("lea");
            let licenses_dir = carl_dir.child("licenses");

            carl_dir.dir_contains_exactly_in_order(vec![
                &lea_dir,
                &licenses_dir,
                &opendut_carl_executable,
            ]);

            opendut_carl_executable.assert_non_empty_file();
            lea_dir.assert(path::is_dir());
            licenses_dir.assert(path::is_dir());

            { //validate license dir contents
                let licenses_index_file = licenses_dir.child("index.json");
                let licenses_carl_file = licenses_dir.child("opendut-carl.licenses.json");
                let licenses_edgar_file = licenses_dir.child("opendut-edgar.licenses.json");
                let licenses_lea_file = licenses_dir.child("opendut-lea.licenses.json");

                licenses_dir.dir_contains_exactly_in_order(vec![
                    &licenses_index_file,
                    &licenses_carl_file,
                    &licenses_edgar_file,
                    &licenses_lea_file,
                ]);

                licenses_index_file.assert(path::is_file());
                let licenses_index_content = fs::read_to_string(licenses_index_file)?;

                for license_file in [&licenses_edgar_file, &licenses_carl_file, &licenses_lea_file] {
                    assert!(
                        licenses_index_content.contains(license_file.file_name_str()),
                        "The license index.json did not contain entry for expected file: {}", license_file.display()
                    );

                    license_file.assert_non_empty_file();
                }
            }

            Ok(())
        }
    }
}
