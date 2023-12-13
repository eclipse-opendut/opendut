use std::fs;
use std::path::PathBuf;
use crate::{Arch, Package};

const PACKAGE: &Package = &Package::Carl;


pub fn build_release(target: &Arch) -> anyhow::Result<()> {
    crate::tasks::build::build_release(PACKAGE, target)?;
    Ok(())
}

pub mod distribution {
    use super::*;

    #[tracing::instrument]
    pub fn carl(target: &Arch) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        let distribution_out_dir = distribution::package_dir(PACKAGE, target);

        distribution::clean()?;

        let build_dir = crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(build_dir, PACKAGE, target)?;

        collect_carl_specific_files(PACKAGE, target, &distribution_out_dir)?;

        distribution::bundle_collected_files(PACKAGE, target)?;

        Ok(())
    }

    #[tracing::instrument]
    pub fn collect_carl_specific_files(package: &Package, target: &Arch, out_dir: &PathBuf) -> anyhow::Result<()> {

        lea::get_lea(&out_dir)?;

        licenses::get_licenses(&out_dir)?;

        Ok(())
    }

    mod lea {
        use super::*;

        #[tracing::instrument]
        pub fn get_lea(out_dir: &PathBuf) -> anyhow::Result<()> {

            let lea_build_dir = crate::packages::lea::distribution::build::build_release()?;
            let lea_out_dir = out_dir.join("lea");

            fs::create_dir_all(&lea_out_dir)?;

            fs_extra::dir::copy(
                &lea_build_dir,
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
        pub fn get_licenses(out_dir: &PathBuf) -> anyhow::Result<()> {

            let carl_licenses = generate_licenses()?;
            let lea_licenses = crate::packages::lea::licenses::generate_licenses()?;
            let edgar_licenses = crate::packages::edgar::licenses::generate_licenses()?;


            let licenses_dir = out_dir.join("licenses");
            fs::create_dir_all(&licenses_dir)?;

            for license_file in [&carl_licenses, &lea_licenses, &edgar_licenses] {
                fs::copy(
                    license_file,
                    licenses_dir.join(license_file.file_name().unwrap())
                )?;
            }

            fs::write(
                &licenses_dir.join("index.json"),
                json!({
                    "carl": carl_licenses.file_name().unwrap().to_str(),
                    "edgar": edgar_licenses.file_name().unwrap().to_str(),
                    "lea": lea_licenses.file_name().unwrap().to_str(),
                }).to_string(),
            )?;

            Ok(())
        }


        pub fn generate_licenses() -> anyhow::Result<PathBuf> {
            crate::tasks::licenses::generate_licenses(PACKAGE)
        }
    }
}
