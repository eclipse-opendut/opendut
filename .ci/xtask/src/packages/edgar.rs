use std::fs;
use std::path::PathBuf;

use anyhow::anyhow;

use crate::{Arch, Package};
use crate::core::types::parsing::arch::ArchSelection;

const PACKAGE: &Package = &Package::Edgar;


#[derive(Debug, clap::Subcommand)]
pub enum EdgarTask {
    /// Perform a release build, without bundling a distribution.
    Build {
        #[arg(long, default_value_t)]
        target: ArchSelection,
    },
    /// Build and bundle a release distribution
    #[command(alias="dist")]
    Distribution {
        #[arg(long, default_value_t)]
        target: ArchSelection,
    },

    /// Download the NetBird Client artifact, as it normally happens when building a distribution.
    /// Intended for parallelization in CI/CD.
    GetNetbirdClientArtifact {
        #[arg(long, default_value_t)]
        target: ArchSelection,
    },
}
impl EdgarTask {
    pub fn handle_task(self) -> anyhow::Result<()> {
        match self {
            EdgarTask::Build { target } => {
                for target in target.iter() {
                    build::build_release(&target)?;
                }
            }
            EdgarTask::Distribution { target } => {
                for target in target.iter() {
                    distribution::edgar_distribution(&target)?;
                }
            }
            EdgarTask::GetNetbirdClientArtifact { target } => {
                for target in target.iter() {
                    distribution::netbird::get_netbird_client_artifact(&target)?;
                }
            }
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
    pub fn edgar_distribution(target: &Arch) -> anyhow::Result<()> {
        use crate::tasks::distribution;

        distribution::clean(PACKAGE, target)?;

        crate::tasks::build::build_release(PACKAGE, target)?;

        distribution::collect_executables(PACKAGE, target)?;

        netbird::get_netbird_client_artifact(target)?;
        distribution::licenses::get_licenses(PACKAGE, target)?;

        distribution::bundle_collected_files(PACKAGE, target)?;

        Ok(())
    }


    pub mod netbird {
        use super::*;

        #[tracing::instrument]
        pub fn get_netbird_client_artifact(target: &Arch) -> anyhow::Result<()> {
            //Modelled after documentation here: https://docs.netbird.io/how-to/getting-started#binary-install

            let metadata = crate::metadata::cargo();
            let version = metadata.workspace_metadata["ci"]["netbird"]["version"].as_str()
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
            fs::create_dir_all(netbird_artifact.parent().unwrap())?;

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
            fs::create_dir_all(out_file.parent().unwrap())?;

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
}
