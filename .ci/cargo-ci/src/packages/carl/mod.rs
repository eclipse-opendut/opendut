use crate::{fs, workspace};
use std::path::Path;
use anyhow::Context;
use tracing::info;
use cicero::distribution::build::Target;
use cicero::distribution::filter::DistributionFilter;
use crate::core::types::parsing::package::PackageSelection;
use crate::packages::carl::distribution::copy_license_json::copy_license_json;
use crate::Package;
use crate::tasks::distribution::bundle;


const SELF_PACKAGE: &Package = &workspace::package::opendut_carl;

/// Tasks available or specific for CARL
#[derive(clap::Parser)]
#[command(alias="opendut-carl")]
pub struct CarlCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(clap::Subcommand)]
pub enum TaskCli {
    Distribution(crate::tasks::distribution::DistributionCli),
    Docker(crate::tasks::docker::DockerCli),
    Licenses(crate::tasks::licenses::LicensesCli),
    Run(crate::tasks::run::RunCli),

    DistributionBuild(crate::tasks::build::DistributionBuildCli),
    DistributionCopyLicenseJson(crate::tasks::distribution::copy_license_json::DistributionCopyLicenseJsonCli),
    DistributionBundleFiles(crate::tasks::distribution::bundle::DistributionBundleFilesCli),
    DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli),

    /// Upload sample data to simplify debugging LEA and CLEO. Run CARL with `cargo carl` beforehand.
    PushSamples,
}

impl CarlCli {
    #[tracing::instrument(name="carl", skip(self))]
    pub fn run(self) -> anyhow::Result<()> {
        match self.task {
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target, release_build }) => {
                build::build_release(target, release_build)?;
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target, release_build }) => {
                distribution::carl_distribution(target, release_build)?;
            }
            TaskCli::Licenses(cli) => cli.run(PackageSelection::Single(SELF_PACKAGE.clone()))?,
            TaskCli::Run(cli) => {
                tracing::info_span!("lea").in_scope(|| {
                    let passthrough =
                        if cli.features.contains(&String::from("viper")) {
                            info!("Running with VIPER enabled.");
                            vec![String::from("--features=viper")]
                        } else {
                            vec![]
                        };

                    let release_build = false;
                    crate::packages::lea::build::build(release_build, passthrough)
                        .context("Error while building LEA for CARL distribution") //ensure the LEA distribution exists and is up-to-date
                })?;

                info!("Starting CARL. You can view the web-UI at: https://localhost:8080");
                cli.run(SELF_PACKAGE)?
            }

            TaskCli::DistributionCopyLicenseJson(cli) => {
                copy_license_json(cli.target, cli.skip_generate.into())?;
            }
            TaskCli::DistributionBundleFiles(cli) => {
                cli.run(SELF_PACKAGE)?;
            }
            TaskCli::DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli { target }) => {
                distribution::validate::validate_contents(target)?;
            }
            TaskCli::Docker(cli) => {
                cli.run(SELF_PACKAGE)?;
            }

            TaskCli::PushSamples => push_samples()?,
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
    use crate::tasks::distribution::copy_license_json::SkipGenerate;

    use super::*;

    #[tracing::instrument]
    pub fn carl_distribution(target: Target, release_build: bool) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        let distribution_out_dir = distribution::out_package_dir(SELF_PACKAGE, target);

        distribution::clean(SELF_PACKAGE, target)?;

        crate::tasks::build::distribution_build(SELF_PACKAGE, target, release_build)?;

        distribution::collect_executables(SELF_PACKAGE, target)?;

        cleo::get_cleo(&distribution_out_dir, release_build)?;
        edgar::get_edgar(&distribution_out_dir, release_build)?;
        lea::get_lea(&distribution_out_dir, release_build)?;
        copy_license_json::copy_license_json(target, SkipGenerate::No)?;

        distribution::bundle::bundle_files(SELF_PACKAGE, target, release_build)?;

        validate::validate_contents(target)?;

        Ok(())
    }

    mod cleo {
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn get_cleo(out_dir: &Path, release_build: bool) -> anyhow::Result<()> {
            let package = workspace::package::opendut_cleo;

            let out_dir = out_dir.join(package.name);
            fs::create_dir_all(&out_dir)?;

            let targets = if release_build {
                crate::packages::cleo::SUPPORTED_TARGETS.to_vec()
            } else {
                vec![Target::default()]
            };

            for target in targets {
                let out_file = out_dir.join(bundle::out_file_name(&package, target));
                crate::packages::cleo::distribution::cleo_distribution(target, &out_file, release_build, DistributionFilter::Disabled)?;
            }

            Ok(())
        }
    }

    mod edgar {
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn get_edgar(out_dir: &Path, release_build: bool) -> anyhow::Result<()> {
            let package = workspace::package::opendut_edgar;

            let out_dir = out_dir.join(package.name);
            fs::create_dir_all(&out_dir)?;

            let targets = if release_build {
                crate::packages::edgar::SUPPORTED_TARGETS.to_vec()
            } else {
                vec![Target::default()]
            };

            for target in targets {
                let out_file = out_dir.join(bundle::out_file_name(&package, target));
                crate::packages::edgar::distribution::edgar_distribution(target, &out_file, release_build, DistributionFilter::Disabled)?;
            }
            Ok(())
        }
    }

    mod lea {
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn get_lea(out_dir: &Path, release_build: bool) -> anyhow::Result<()> {

            let passthrough = vec![];
            crate::packages::lea::build::build(release_build, passthrough)?;
            let lea_build_dir = crate::packages::lea::build::out_dir();

            let lea_out_dir = out_dir.join(workspace::package::opendut_lea.name);
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
        use tracing::info;

        use crate::tasks::distribution::copy_license_json::SkipGenerate;

        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn copy_license_json(target: Target, skip_generate: SkipGenerate) -> anyhow::Result<()> {

            match skip_generate {
                SkipGenerate::Yes => info!("Skipping generation of licenses, as requested. Directly attempting to copy to target location."),
                SkipGenerate::No => {
                    use workspace::package::*;
                    for package in [SELF_PACKAGE, &opendut_lea, &opendut_edgar, &opendut_cleo] {
                        crate::tasks::licenses::json::export_json(package)?;
                    }
                }
            };

            let carl_in_file = crate::tasks::licenses::json::out_file(SELF_PACKAGE);
            let carl_out_file = crate::tasks::distribution::copy_license_json::out_file(SELF_PACKAGE, target);
            let out_dir = carl_out_file.parent().unwrap();

            let cleo_in_file = crate::tasks::licenses::json::out_file(&workspace::package::opendut_cleo);
            let cleo_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(&workspace::package::opendut_cleo));
            let lea_in_file = crate::tasks::licenses::json::out_file(&workspace::package::opendut_lea);
            let lea_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(&workspace::package::opendut_lea));
            let edgar_in_file = crate::tasks::licenses::json::out_file(&workspace::package::opendut_edgar);
            let edgar_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(&workspace::package::opendut_edgar));

            fs::create_dir_all(out_dir)?;
            fs::copy(carl_in_file, &carl_out_file)?;
            fs::copy(cleo_in_file, &cleo_out_file)?;
            fs::copy(lea_in_file, &lea_out_file)?;
            fs::copy(edgar_in_file, &edgar_out_file)?;

            fs::write(
                out_dir.join("index.json"),
                json!({
                    "carl": carl_out_file.file_name().unwrap().to_str(),
                    "edgar": edgar_out_file.file_name().unwrap().to_str(),
                    "cleo": cleo_out_file.file_name().unwrap().to_str(),
                    "lea": lea_out_file.file_name().unwrap().to_str(),
                }).to_string(),
            )?;

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


            let carl_dir = unpack_dir.child(SELF_PACKAGE.name);
            carl_dir.assert(path::is_dir());

            let opendut_carl_executable = carl_dir.child(SELF_PACKAGE.name);
            let opendut_cleo_dir = carl_dir.child(workspace::package::opendut_cleo.name);
            let opendut_edgar_dir = carl_dir.child(workspace::package::opendut_edgar.name);
            let opendut_lea_dir = carl_dir.child(workspace::package::opendut_lea.name);
            let licenses_dir = carl_dir.child("licenses");

            carl_dir.dir_contains_exactly_in_order(vec![
                &licenses_dir,
                &opendut_carl_executable,
                &opendut_cleo_dir,
                &opendut_edgar_dir,
                &opendut_lea_dir,
            ]);

            opendut_carl_executable.assert_non_empty_file();
            opendut_cleo_dir.assert(path::is_dir());
            opendut_edgar_dir.assert(path::is_dir());
            opendut_lea_dir.assert(path::is_dir());
            licenses_dir.assert(path::is_dir());

            { //validate license dir contents
                let licenses_index_file = licenses_dir.child("index.json");
                let licenses_carl_file = licenses_dir.child("opendut-carl.licenses.json");
                let licenses_edgar_file = licenses_dir.child("opendut-edgar.licenses.json");
                let licenses_cleo_file = licenses_dir.child("opendut-cleo.licenses.json");
                let licenses_lea_file = licenses_dir.child("opendut-lea.licenses.json");

                licenses_dir.dir_contains_exactly_in_order(vec![
                    &licenses_index_file,
                    &licenses_carl_file,
                    &licenses_cleo_file,
                    &licenses_edgar_file,
                    &licenses_lea_file,
                ]);

                licenses_index_file.assert(path::is_file());
                let licenses_index_content = fs::read_to_string(licenses_index_file)?;

                for license_file in [&licenses_edgar_file, &licenses_carl_file, &licenses_cleo_file, &licenses_lea_file] {
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


fn push_samples() -> anyhow::Result<()> {
    let file = std::env::temp_dir().join("opendut-cargo-ci-carl-push-samples.yaml");

    fs::write(&file, include_str!("cargo-ci-carl-push-samples.yaml"))?;

    crate::tasks::run::RunCli {
        features: vec![],
        passthrough: vec!["apply".to_string(), file.display().to_string()],
    }
    .run(&workspace::package::opendut_cleo)?;

    Ok(())
}
