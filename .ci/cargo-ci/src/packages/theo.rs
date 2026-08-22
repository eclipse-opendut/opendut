use cicero::distribution::build::Target;

use crate::{Package, workspace};
use crate::core::types::parsing::package::PackageSelection;
use crate::tasks::build::BuildArgs;

const SELF_PACKAGE: &Package = &workspace::package::opendut_theo;


/// Tasks available or specific for THEO
#[derive(clap::Parser)]
#[command(alias="opendut-theo")]
pub struct TheoCli {
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

impl TheoCli {
    #[tracing::instrument(name="theo", skip(self))]
    pub fn run(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target, build_args }) => {
                build::build_release(target, &build_args)?;
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target, build_args }) => {
                distribution::theo_distribution(target, &build_args)?;
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

    #[tracing::instrument(skip_all)]
    pub fn theo_distribution(target: Target, build_args: &BuildArgs) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        distribution::clean(SELF_PACKAGE, target)?;

        crate::tasks::build::distribution_build(SELF_PACKAGE, target, build_args)?;

        distribution::collect_executables(SELF_PACKAGE, target)?;

        distribution::copy_license_json::copy_license_json(SELF_PACKAGE, target, SkipGenerate::No)?;

        distribution::bundle::bundle_files(SELF_PACKAGE, target, build_args.release_build)?;

        validate::validate_contents(target)?;

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

        #[tracing::instrument]
        pub fn validate_contents(target: Target) -> anyhow::Result<()> {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let archive = bundle::out_file(SELF_PACKAGE, target);
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };

            let theo_dir = unpack_dir.child(SELF_PACKAGE.name);
            theo_dir.assert(path::is_dir());

            let opendut_edgar_executable = theo_dir.child(SELF_PACKAGE.name);
            let licenses_dir = theo_dir.child("licenses");

            theo_dir.dir_contains_exactly_in_order(vec![
                &licenses_dir,
                &opendut_edgar_executable,
            ]);

            opendut_edgar_executable.assert_non_empty_file();
            licenses_dir.assert(path::is_dir());

            {   //validate licenses dir contents
                let licenses_edgar_file = licenses_dir.child("opendut-theo.licenses.json");

                licenses_dir.dir_contains_exactly_in_order(vec![
                    &licenses_edgar_file,
                ]);

                licenses_edgar_file.assert_non_empty_file();
            }

            Ok(())
        }
    }
}
