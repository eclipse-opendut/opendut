use crate::fs;
use std::path::{Path, PathBuf};
use anyhow::Context;
use tracing::info;
use crate::core::types::parsing::package::PackageSelection;
use crate::packages::carl::distribution::copy_license_json::copy_license_json;
use crate::{Arch, Package};

const SELF_PACKAGE: Package = Package::Carl;

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
}

impl CarlCli {
    #[tracing::instrument(name="carl", skip(self))]
    pub fn default_handling(self) -> crate::Result {
        match self.task {
            TaskCli::DistributionBuild(crate::tasks::build::DistributionBuildCli { target }) => {
                for target in target.iter() {
                    build::build_release(target)?;
                }
            }
            TaskCli::Distribution(crate::tasks::distribution::DistributionCli { target }) => {
                for target in target.iter() {
                    distribution::carl_distribution(target)?;
                }
            }
            TaskCli::Licenses(cli) => cli.default_handling(PackageSelection::Single(SELF_PACKAGE))?,
            TaskCli::Run(cli) => {
                tracing::info_span!("lea").in_scope(|| {
                    crate::packages::lea::build::build()
                        .context("Error while building LEA for CARL distribution") //ensure the LEA distribution exists and is up-to-date
                })?;

                info!("Starting CARL. You can view the web-UI at: https://localhost:8080");
                cli.default_handling(SELF_PACKAGE)?
            }

            TaskCli::DistributionCopyLicenseJson(implementation) => {
                for target in implementation.target.iter() {
                    copy_license_json(target, implementation.skip_generate.into())?;
                }
            }
            TaskCli::DistributionBundleFiles(implementation) => {
                implementation.default_handling(SELF_PACKAGE)?;
            }
            TaskCli::DistributionValidateContents(crate::tasks::distribution::validate::DistributionValidateContentsCli { target }) => {
                for target in target.iter() {
                    distribution::validate::validate_contents(target)?;
                }
            }
            TaskCli::Docker(crate::tasks::docker::DockerCli { tag, publish }) => {
                crate::tasks::docker::build_carl_docker_image(tag.clone())?;
                if publish {
                    crate::tasks::docker::publish_carl_docker_image(tag)?;
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
    pub fn out_dir(target: Arch) -> PathBuf {
        crate::tasks::build::out_dir(SELF_PACKAGE, target)
    }
}

pub mod distribution {
    use crate::tasks::distribution::copy_license_json::SkipGenerate;

    use super::*;

    #[tracing::instrument]
    pub fn carl_distribution(target: Arch) -> crate::Result {
        use crate::tasks::distribution;

        let distribution_out_dir = distribution::out_package_dir(SELF_PACKAGE, target);

        distribution::clean(SELF_PACKAGE, target)?;

        crate::tasks::build::distribution_build(SELF_PACKAGE, target)?;

        distribution::collect_executables(SELF_PACKAGE, target)?;

        cleo::get_cleo(&distribution_out_dir)?;
        edgar::get_edgar(&distribution_out_dir)?;
        lea::get_lea(&distribution_out_dir)?;
        copy_license_json::copy_license_json(target, SkipGenerate::No)?;

        distribution::bundle::bundle_files(SELF_PACKAGE, target)?;

        validate::validate_contents(target)?;

        Ok(())
    }

    mod cleo {
        use anyhow::Context;

        use crate::tasks::distribution::bundle;

        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn get_cleo(out_dir: &Path) -> crate::Result {

            let cleo_out_dir = out_dir.join(Package::Cleo.ident());
            fs::create_dir_all(cleo_out_dir)?;
            
            for arch in crate::packages::cleo::SUPPORTED_ARCHITECTURES {
                crate::packages::cleo::distribution::cleo_distribution(arch.to_owned())?;
                let cleo_build_dir = crate::tasks::distribution::out_arch_dir(arch.to_owned());

                let cleo_arch_dir = out_dir.join(Package::Cleo.ident());
                fs::create_dir_all(&cleo_arch_dir)?;

                let tar_file_name = bundle::out_file(Package::Cleo, arch);

                let cleo_tar_file_name = tar_file_name.file_name().context(format!("Could not extract file name {}", &tar_file_name.display()))?;

                fs_extra::file::copy(
                    cleo_build_dir.join(&tar_file_name),
                    cleo_arch_dir.join(cleo_tar_file_name),
                    &fs_extra::file::CopyOptions::default()
                        .overwrite(true)
                )?;
            }

            Ok(())
        }
    }

    mod edgar {
        use anyhow::Context;

        use crate::tasks::distribution::bundle;

        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn get_edgar(out_dir: &Path) -> crate::Result {

            let edgar_out_dir = out_dir.join(Package::Edgar.ident());
            fs::create_dir_all(edgar_out_dir)?;

            for arch in crate::packages::edgar::SUPPORTED_ARCHITECTURES {
                crate::packages::edgar::distribution::edgar_distribution(arch.to_owned())?;
                let edgar_build_dir = crate::tasks::distribution::out_arch_dir(arch.to_owned());

                let edgar_arch_dir = out_dir.join(Package::Edgar.ident());
                fs::create_dir_all(&edgar_arch_dir)?;

                let tar_file_name = bundle::out_file(Package::Edgar, arch);

                let edgar_tar_file_name = tar_file_name.file_name().context(format!("Could not extract file name {}", &tar_file_name.display()))?;

                fs_extra::file::copy(
                    edgar_build_dir.join(&tar_file_name),
                    edgar_arch_dir.join(edgar_tar_file_name),
                    &fs_extra::file::CopyOptions::default()
                        .overwrite(true)
                )?;
            }

            Ok(())
        }
    }

    mod lea {
        use super::*;

        #[tracing::instrument(skip_all)]
        pub fn get_lea(out_dir: &Path) -> crate::Result {

            crate::packages::lea::build::build()?;
            let lea_build_dir = crate::packages::lea::build::out_dir();

            let lea_out_dir = out_dir.join(Package::Lea.ident());

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
        pub fn copy_license_json(target: Arch, skip_generate: SkipGenerate) -> crate::Result {

            match skip_generate {
                SkipGenerate::Yes => info!("Skipping generation of licenses, as requested. Directly attempting to copy to target location."),
                SkipGenerate::No => {
                    for package in [SELF_PACKAGE, Package::Lea, Package::Edgar, Package::Cleo] {
                        crate::tasks::licenses::json::export_json(package)?;
                    }
                }
            };

            let carl_in_file = crate::tasks::licenses::json::out_file(SELF_PACKAGE);
            let carl_out_file = crate::tasks::distribution::copy_license_json::out_file(SELF_PACKAGE, target);
            let out_dir = carl_out_file.parent().unwrap();

            let cleo_in_file = crate::tasks::licenses::json::out_file(Package::Cleo);
            let cleo_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(Package::Cleo));
            let lea_in_file = crate::tasks::licenses::json::out_file(Package::Lea);
            let lea_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(Package::Lea));
            let edgar_in_file = crate::tasks::licenses::json::out_file(Package::Edgar);
            let edgar_out_file = out_dir.join(crate::tasks::licenses::json::out_file_name(Package::Edgar));

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
        pub fn validate_contents(target: Arch) -> crate::Result {

            let unpack_dir = {
                let unpack_dir = assert_fs::TempDir::new()?;
                let archive = bundle::out_file(SELF_PACKAGE, target);
                let mut archive = tar::Archive::new(GzDecoder::new(File::open(archive)?));
                archive.set_preserve_permissions(true);
                archive.unpack(&unpack_dir)?;
                unpack_dir
            };


            let carl_dir = unpack_dir.child(SELF_PACKAGE.ident());
            carl_dir.assert(path::is_dir());

            let opendut_carl_executable = carl_dir.child(SELF_PACKAGE.ident());
            let opendut_cleo_dir = carl_dir.child(Package::Cleo.ident());
            let opendut_edgar_dir = carl_dir.child(Package::Edgar.ident());
            let opendut_lea_dir = carl_dir.child(Package::Lea.ident());
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
