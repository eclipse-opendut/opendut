use std::fs;
use std::path::PathBuf;
use crate::{Arch, Package};

const PACKAGE: &Package = &Package::Carl;

/// Tasks available or specific for CARL
#[derive(Debug, clap::Parser)]
#[command(alias="opendut-carl")]
pub struct CarlCli {
    #[command(subcommand)]
    pub task: Task,
}

#[derive(Debug, clap::Subcommand)]
pub enum Task {
    Build(crate::tasks::build::Build),
    Distribution(crate::tasks::distribution::Distribution),
}

impl CarlCli {
    pub fn handle(self) -> anyhow::Result<()> {
        match self.task {
            Task::Build(crate::tasks::build::Build { target }) => {
                for target in target.iter() {
                    build::build_release(&target)?;
                }
            },
            Task::Distribution(crate::tasks::distribution::Distribution { target }) => {
                for target in target.iter() {
                    distribution::carl_distribution(&target)?;
                }
            },
        };
        Ok(())
    }
}

pub mod build {
    use super::*;

    pub fn build_release(target: &Arch) -> anyhow::Result<()> {
        crate::tasks::build::build_release(PACKAGE, target)
    }
    pub fn out_dir(target: &Arch) -> PathBuf {
        crate::tasks::build::out_dir(PACKAGE, target)
    }
}

pub mod distribution {
    use super::*;

    #[tracing::instrument]
    pub fn carl_distribution(target: &Arch) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        let distribution_out_dir = distribution::out_package_dir(PACKAGE, target);

        distribution::clean(PACKAGE, target)?;

        crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(PACKAGE, target)?;

        lea::get_lea(&distribution_out_dir)?;
        licenses::get_licenses(target)?;

        distribution::bundle_collected_files(PACKAGE, target)?;

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

    mod licenses {
        use super::*;
        use serde_json::json;

        #[tracing::instrument]
        pub fn get_licenses(target: &Arch) -> anyhow::Result<()> {

            crate::tasks::distribution::licenses::get_licenses(PACKAGE, target)?;
            let carl_licenses_file = crate::tasks::distribution::licenses::out_file(PACKAGE, target);

            crate::tasks::distribution::licenses::get_licenses(&Package::Lea, target)?;
            let lea_licenses_file = crate::tasks::distribution::licenses::out_file(&Package::Lea, target);

            crate::tasks::distribution::licenses::get_licenses(&Package::Edgar, target)?;
            let edgar_licenses_file = crate::tasks::distribution::licenses::out_file(&Package::Edgar, target);


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
