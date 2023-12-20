use std::fs;
use std::path::PathBuf;
use crate::{Target, Package};
use crate::core::types::parsing::package::PackageSelection;

const PACKAGE: &Package = &Package::Carl;

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
    DistributionCopyLicenseJson(crate::tasks::distribution::copy_license_json::CopyLicenseJsonCli),
    #[command(hide=true)]
    DistributionBundleFiles(crate::tasks::distribution::bundle::DistributionBundleFilesCli),
}

impl CarlCli {
    pub fn handle(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::Build(crate::tasks::build::BuildCli { target }) => {
                for target in target.iter() {
                    build::build_release(&target)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target }) => {
                for target in target.iter() {
                    distribution::carl_distribution(&target)?;
                }
            }
            TaskCli::Licenses(implementation) => {
                implementation.handle(PackageSelection::Single(*PACKAGE))?;
            }

            TaskCli::DistributionCopyLicenseJson(implementation) => {
                implementation.handle(PACKAGE)?;
            }
            TaskCli::DistributionBundleFiles(implementation) => {
                implementation.handle(PACKAGE)?;
            }
        };
        Ok(())
    }
}

pub mod build {
    use super::*;

    pub fn build_release(target: &Target) -> anyhow::Result<()> {
        crate::tasks::build::build_release(PACKAGE, target)
    }
    pub fn out_dir(target: &Target) -> PathBuf {
        crate::tasks::build::out_dir(PACKAGE, target)
    }
}

pub mod distribution {
    use crate::tasks::distribution::copy_license_json::SkipGenerate;
    use super::*;

    #[tracing::instrument]
    pub fn carl_distribution(target: &Target) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        let distribution_out_dir = distribution::out_package_dir(PACKAGE, target);

        distribution::clean(PACKAGE, target)?;

        crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(PACKAGE, target)?;

        lea::get_lea(&distribution_out_dir)?;
        copy_license_json::copy_license_json(target, SkipGenerate::No)?;

        distribution::bundle::bundle_files(PACKAGE, target)?;

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

    mod copy_license_json {
        use super::*;
        use serde_json::json;
        use crate::tasks::distribution::copy_license_json::SkipGenerate;

        #[tracing::instrument]
        pub fn copy_license_json(target: &Target, skip_generate: SkipGenerate) -> anyhow::Result<()> {

            crate::tasks::distribution::copy_license_json::copy_license_json(PACKAGE, target, skip_generate)?;
            let carl_licenses_file = crate::tasks::distribution::copy_license_json::out_file(PACKAGE, target);

            crate::tasks::distribution::copy_license_json::copy_license_json(&Package::Lea, target, skip_generate)?;
            let lea_licenses_file = crate::tasks::distribution::copy_license_json::out_file(&Package::Lea, target);

            crate::tasks::distribution::copy_license_json::copy_license_json(&Package::Edgar, target, skip_generate)?;
            let edgar_licenses_file = crate::tasks::distribution::copy_license_json::out_file(&Package::Edgar, target);


            let out_dir = crate::tasks::distribution::out_package_dir(PACKAGE, target);
            let licenses_dir = out_dir.join("licenses");
            fs::create_dir_all(&licenses_dir)?;

            for license_file in [&carl_licenses_file, &lea_licenses_file, &edgar_licenses_file] {
                fs::copy(
                    license_file,
                    licenses_dir.join(license_file.file_name().unwrap())
                )?;
            }

            fs::write(
                licenses_dir.join("index.json"),
                json!({
                    "carl": carl_licenses_file.file_name().unwrap().to_str(),
                    "edgar": edgar_licenses_file.file_name().unwrap().to_str(),
                    "lea": lea_licenses_file.file_name().unwrap().to_str(),
                }).to_string(),
            )?;

            Ok(())
        }
    }
}
