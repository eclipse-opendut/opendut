use std::path::PathBuf;
use crate::Package;

const PACKAGE: &Package = &Package::OpendutLea;


pub mod distribution {

    pub mod build {
        use std::path::PathBuf;
        use std::process::Command;

        #[tracing::instrument]
        pub fn build_release() -> anyhow::Result<PathBuf> {
            crate::util::install_crate("trunk")?;

            let working_dir = crate::packages::opendut_lea::self_dir();
            let target_dir = working_dir.join("dist");

            Command::new("trunk")
                .args([
                    "build",
                    "--release",
                    "--dist", &target_dir.display().to_string(),
                ])
                .current_dir(working_dir)
                .status()?;

            Ok(target_dir)
        }
    }
}

pub mod licenses {
    use super::*;

    pub fn generate_licenses() -> anyhow::Result<PathBuf> {
        crate::tasks::licenses::generate_licenses(PACKAGE)
    }
}

pub fn self_dir() -> PathBuf {
    crate::constants::workspace_dir()
        .join(crate::Package::OpendutLea.ident())
}
