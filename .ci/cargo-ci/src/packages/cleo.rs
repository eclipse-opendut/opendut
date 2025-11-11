use std::path::PathBuf;

use crate::{Arch, Package};
use crate::core::types::parsing::package::PackageSelection;

pub const SUPPORTED_ARCHITECTURES: [Arch; 3] = [Arch::X86_64, Arch::Armhf, Arch::Arm64];

const SELF_PACKAGE: Package = Package::Cleo;


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
    pub fn default_handling(self) -> crate::Result {
        match self.task {
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target, release_build }) => {
                for target in target.iter() {
                    build::build_release(target, release_build)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target, release_build }) => {
                for target in target.iter() {
                    distribution::cleo_distribution(target, release_build)?;
                }
            }
            TaskCli::Licenses(cli) => cli.default_handling(PackageSelection::Single(SELF_PACKAGE))?,
            TaskCli::Run(cli) => cli.default_handling(SELF_PACKAGE)?,

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

    pub fn build_release(target: Arch, release_build: bool) -> crate::Result {
        crate::tasks::build::distribution_build(SELF_PACKAGE, target, release_build)
    }
    pub fn out_dir(target: Arch, release_build: bool) -> PathBuf {
        crate::tasks::build::out_file(SELF_PACKAGE, target, release_build)
    }
}

pub mod distribution {
    use crate::tasks::distribution::copy_license_json::SkipGenerate;
    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn cleo_distribution(target: Arch, release_build: bool) -> crate::Result {
        use crate::tasks::distribution;

        crate::tasks::build::distribution_build(SELF_PACKAGE, target, release_build)?;

        cicero::cache::Output::from(
            distribution::bundle::out_file(SELF_PACKAGE, target)
        ).rebuild_on_change(
            [crate::tasks::build::out_file(SELF_PACKAGE, target, release_build)],
            || {

                distribution::clean(SELF_PACKAGE, target)?;

                distribution::collect_executables(SELF_PACKAGE, target, release_build)?;

                distribution::copy_license_json::copy_license_json(SELF_PACKAGE, target, SkipGenerate::No)?;

                distribution::bundle::bundle_files(SELF_PACKAGE, target)?;

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
        pub fn validate_contents(target: Arch) -> crate::Result {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let archive = bundle::out_file(SELF_PACKAGE, target);
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };

            let cleo_dir = unpack_dir.child(SELF_PACKAGE.ident());
            cleo_dir.assert(path::is_dir());

            let opendut_edgar_executable = cleo_dir.child(SELF_PACKAGE.ident());
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
