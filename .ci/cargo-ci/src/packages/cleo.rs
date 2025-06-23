use cicero::distribution::build::{target, Target};

use crate::fs;
use crate::{Package, workspace};
use crate::core::types::parsing::package::PackageSelection;

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
    Distribution(crate::tasks::distribution::DistributionCliWithFilter),
    Licenses(crate::tasks::licenses::LicensesCli),
    Run(crate::tasks::run::RunCli),
}

impl CleoCli {
    #[tracing::instrument(name="cleo", skip(self))]
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

                distribution::cleo_distribution(target, &out_file, release_build, filter)?;
            }
            TaskCli::Licenses(cli) => cli.run(PackageSelection::Single(SELF_PACKAGE.clone()))?,
            TaskCli::Run(cli) => cli.run(SELF_PACKAGE)?,
        };
        Ok(())
    }
}


pub mod distribution {
    use super::*;
    use std::path::Path;
    use cicero::distribution::{Distribution, DistributionOptions, bundle::tar::TarBundler, filter::DistributionFilter};

    #[tracing::instrument(skip_all)]
    pub fn cleo_distribution(target: Target, out_file: &Path, release_build: bool, filter: DistributionFilter) -> anyhow::Result<()> {

        let distribution = Distribution::new_with_options(
            "opendut-cleo",
            DistributionOptions { filter: filter.clone() },
        )?;

        distribution.add_file("opendut-cleo", |out_file| {
            crate::tasks::build::distribution_build_with_out_path(SELF_PACKAGE, target, out_file, release_build)
        })?;

        distribution.dir("licenses")?
            .add_file("opendut-cleo.licenses.json", |out_file| crate::tasks::licenses::json::export_json_with_out_path(SELF_PACKAGE, out_file))?;

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

    pub mod validate {
        use std::path::Path;
        use crate::fs::File;

        use assert_fs::prelude::*;
        use flate2::read::GzDecoder;
        use predicates::path;

        use crate::core::util::file::ChildPathExt;

        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn validate_contents_of(path: &Path) -> anyhow::Result<()> {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(path)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };

            let cleo_dir = unpack_dir.child(SELF_PACKAGE.name);
            cleo_dir.assert(path::is_dir());

            let executable = cleo_dir.child(SELF_PACKAGE.name);
            let licenses_dir = cleo_dir.child("licenses");

            cleo_dir.dir_contains_exactly_in_order(vec![
                &licenses_dir,
                &executable,
            ]);

            executable.assert_non_empty_file();
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
