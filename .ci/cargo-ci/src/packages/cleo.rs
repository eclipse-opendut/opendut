use crate::fs;
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
    Distribution(crate::tasks::distribution::DistributionCliWithFilter),
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
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target }) => {
                for target in target.iter() {
                    build::build_release(target)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCliWithFilter { target, filter }) => {
                let filter = if filter.is_empty() {
                    cicero::distribution::filter::DistributionFilter::Disabled
                } else {
                    cicero::distribution::filter::DistributionFilter::Enabled(filter)
                };

                for target in target.iter() {
                    let out_file = crate::tasks::distribution::bundle::out_file(SELF_PACKAGE, target);
                    distribution::cleo_distribution(target, &out_file, filter.clone())?;
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

    pub fn build_release(target: Arch) -> crate::Result {
        crate::tasks::build::distribution_build(SELF_PACKAGE, target)
    }
}

pub mod distribution {
    use std::path::Path;
    use cicero::distribution::{filter::DistributionFilter, Distribution, DistributionOptions};
    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn cleo_distribution(target: Arch, out_file: &Path, filter: DistributionFilter) -> crate::Result {

        let distribution = Distribution::new_with_options(
            format!("opendut-cleo-{target}"),
            DistributionOptions { filter: filter.clone() }, //TODO don't clone?
        )?;

        distribution.add_file("opendut-cleo", |out_file| {
            crate::tasks::build::distribution_build_with_out_path(SELF_PACKAGE, target, out_file)
        })?;

        distribution.dir("licenses")?
            .add_file("opendut-cleo.licenses.json", |out_file| crate::tasks::licenses::json::export_json_with_out_path(SELF_PACKAGE, out_file))?;

        let distribution_path =
            if let DistributionFilter::Disabled = filter {
                let distribution_path = distribution.bundle_as_tar_gz()?;
                validate::validate_contents_of(&distribution_path, target)?;
                distribution_path
            } else {
                distribution.bundle_as_dir()?
            };

        fs::create_dir_all(out_file.parent().unwrap())?;
        fs::rename(distribution_path, out_file)?;

        Ok(())
    }

    pub mod validate {
        use std::path::Path;
        use crate::fs::File;

        use assert_fs::prelude::*;
        use flate2::read::GzDecoder;
        use predicates::path;

        use crate::core::util::file::ChildPathExt;
        use crate::tasks::distribution::bundle;

        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn validate_contents(target: Arch) -> crate::Result {
            let out_file = bundle::out_file(SELF_PACKAGE, target);
            validate_contents_of(&out_file, target)
        }

        #[tracing::instrument(skip_all)]
        pub fn validate_contents_of(path: &Path, target: Arch) -> crate::Result {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(path)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };

            let cleo_dir = unpack_dir.child(format!("{SELF_PACKAGE}-{target}"));
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
