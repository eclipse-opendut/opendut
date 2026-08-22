use cicero::distribution::build::{target, Target};

use crate::{Package, workspace};
use crate::core::types::parsing::package::PackageSelection;
use crate::tasks::build::BuildArgs;

pub const SUPPORTED_TARGETS: [Target; 3] = [target::x86_64_unknown_linux_gnu, target::armv7_unknown_linux_gnueabihf, target::aarch64_unknown_linux_gnu];

const SELF_PACKAGE: &Package = &workspace::package::opendut_cleo;


/// Tasks available or specific for CLEO
#[derive(clap::Parser)]
#[command(alias="opendut-cleo")]
pub struct CleoCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    Distribution(crate::tasks::distribution::DistributionCli),
    Licenses(crate::tasks::licenses::LicensesCli),
    Run(crate::tasks::run::RunCli),

    DistributionBuild(crate::tasks::build::DistributionBuildCli),
    DistributionCopyLicenseJson(crate::tasks::distribution::copy_license_json::DistributionCopyLicenseJsonCli),
    DistributionBundleFiles(crate::tasks::distribution::bundle::DistributionBundleFilesCli),
    DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli),
}

impl CleoCli {
    #[tracing::instrument(name="cleo", skip(self))]
    pub fn run(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target, build_args }) => {
                build::build_release(target, &build_args)?;
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target, build_args }) => {
                distribution::cleo_distribution(target, &build_args)?;
            }
            TaskCli::Licenses(cli) => cli.run(PackageSelection::Single(SELF_PACKAGE.clone()))?,
            TaskCli::Run(cli) => cli.run(SELF_PACKAGE)?,

            TaskCli::DistributionCopyLicenseJson(cli) => cli.run(SELF_PACKAGE)?,
            TaskCli::DistributionBundleFiles(cli) => cli.run(SELF_PACKAGE)?,
            TaskCli::DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli { target }) => {
                distribution::validate::validate_contents(target)?;
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
    pub fn cleo_distribution(target: Target, build_args: &BuildArgs) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        crate::tasks::build::distribution_build(SELF_PACKAGE, target, build_args)?;

        cicero::cache::Output::from(
            distribution::bundle::out_file(SELF_PACKAGE, target)
        ).rebuild_on_change(
            [crate::tasks::build::out_file(SELF_PACKAGE, target)],
            || {

                distribution::clean(SELF_PACKAGE, target)?;

                distribution::collect_executables(SELF_PACKAGE, target)?;

                distribution::copy_license_json::copy_license_json(SELF_PACKAGE, target, SkipGenerate::No)?;

                distribution::bundle::bundle_files(SELF_PACKAGE, target, build_args.release_build)?;

                validate::validate_contents(target)?;

                Ok(())
            })?;

        Ok(())
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

            let cleo_dir = unpack_dir.child(SELF_PACKAGE.name);
            cleo_dir.assert(path::is_dir());

            let opendut_edgar_executable = cleo_dir.child(SELF_PACKAGE.name);
            let licenses_dir = cleo_dir.child("licenses");

            cleo_dir.dir_contains_exactly_in_order(vec![
                &licenses_dir,
                &opendut_edgar_executable,
            ]);

            opendut_edgar_executable.assert_non_empty_file();
            licenses_dir.assert(path::is_dir());

            {   //validate licenses dir contents
                let licenses_edgar_file = licenses_dir.child("opendut-cleo.licenses.json");

                licenses_dir.dir_contains_exactly_in_order(vec![
                    &licenses_edgar_file,
                ]);

                licenses_edgar_file.assert_non_empty_file();
            }

            Ok(())
        }
    }
}
