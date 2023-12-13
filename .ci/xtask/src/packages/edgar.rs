use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;

use crate::{Arch, Package};

const PACKAGE: &Package = &Package::Edgar;


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
    pub fn edgar(target: &Arch) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        distribution::clean(PACKAGE, target)?;

        crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(PACKAGE, target)?;

        collect_edgar_specific_files(target)?;

        distribution::bundle_collected_files(PACKAGE, target)?;

        Ok(())
    }


    #[tracing::instrument]
    fn collect_edgar_specific_files(target: &Arch) -> anyhow::Result<()> {

        netbird::get_netbird_client_artifact(target)?;

        licenses::get_licenses(target)?;

        Ok(())
    }


    mod netbird {
        use super::*;

        #[tracing::instrument]
        pub fn get_netbird_client_artifact(target: &Arch) -> anyhow::Result<()> {
            //Modelled after documentation here: https://docs.netbird.io/how-to/getting-started#binary-install

            let metadata = crate::metadata::cargo();
            let version = metadata.workspace_metadata["ci"]["netbird"]["version"].as_str().clone()
                .ok_or(anyhow!("NetBird version not defined."))?;

            let os = "linux";

            let arch = match target {
                Arch::X86_64 => "amd64",
                Arch::Arm64 => "arm64",
                Arch::Armhf => "armv6",
            };

            let folder_name = format!("v{version}");
            let file_name = format!("netbird_{version}_{os}_{arch}.tar.gz");

            let netbird_artifact = download_dir().join(&folder_name).join(&file_name);
            fs::create_dir_all(&netbird_artifact.parent().unwrap())?;

            if !netbird_artifact.exists() { //download
                let url = format!("https://github.com/reimarstier/netbird/releases/download/{folder_name}/{file_name}");

                println!("Downloading netbird_{version}_{os}_{arch}.tar.gz...");
                let bytes = reqwest::blocking::get(url)?
                    .error_for_status()?
                    .bytes()?;
                println!("Retrieved {} bytes.", bytes.len());

                fs::write(&netbird_artifact, bytes)
                    .map_err(|cause| anyhow!("Error while writing to '{}': {cause}", netbird_artifact.display()))?;
            }
            assert!(netbird_artifact.exists());

            let out_file = out_file(PACKAGE, target);
            fs::create_dir_all(&out_file.parent().unwrap())?;

            fs::copy(&netbird_artifact, &out_file)
                .map_err(|cause| anyhow!("Error while copying from '{}' to '{}': {cause}", netbird_artifact.display(), out_file.display()))?;

            Ok(())
        }

        fn download_dir() -> PathBuf {
            crate::constants::target_dir().join("netbird")
        }

        pub fn out_file(package: &Package, target: &Arch) -> PathBuf {
            crate::tasks::distribution::out_package_dir(package, target).join("install").join("netbird.tar.gz")
        }
    }

    pub mod licenses {
        use super::*;

        #[tracing::instrument]
        pub fn get_licenses(target: &Arch) -> anyhow::Result<()> {

            crate::packages::edgar::licenses::generate_licenses()?;
            let licenses_file = crate::packages::edgar::licenses::out_file();

            let out_dir = out_dir(target);
            let licenses_file_name = format!("{}.licenses.json", PACKAGE.ident());
            fs::create_dir_all(&out_dir)?;

            fs::copy(
                &licenses_file,
                &out_dir.join(&licenses_file_name)
            )?;

            Ok(())
        }
        fn out_dir(target: &Arch) -> PathBuf {
            crate::tasks::distribution::out_package_dir(PACKAGE, target).join("licenses")
        }
    }
}

pub mod licenses {
    use super::*;

    pub fn generate_licenses() -> anyhow::Result<()> {
        crate::tasks::licenses::generate_licenses(PACKAGE)
    }
    pub fn out_file() -> PathBuf {
        crate::tasks::licenses::out_file(PACKAGE)
    }
}
