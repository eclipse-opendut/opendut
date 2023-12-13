use std::path::PathBuf;
use std::process::Command;

use crate::Package;

const PACKAGE: &Package = &Package::Lea;


pub mod build {
    use super::*;

    #[tracing::instrument]
    pub fn build_release() -> anyhow::Result<()> {
        crate::util::install_crate("trunk")?;

        let working_dir = crate::packages::lea::self_dir();
        let out_dir = out_dir();

        Command::new("trunk")
            .args([
                "build",
                "--release",
                "--dist", &out_dir.display().to_string(),
            ])
            .current_dir(working_dir)
            .status()?;

        Ok(())
    }
    pub fn out_dir() -> PathBuf {
        crate::packages::lea::self_dir().join("dist")
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

#[tracing::instrument]
pub fn lea_watch() -> anyhow::Result<()> {
    crate::util::install_crate("trunk")?;

    Command::new("trunk")
        .arg("watch")
        .current_dir(self_dir())
        .status()?;

    Ok(())
}

pub fn self_dir() -> PathBuf {
    crate::constants::workspace_dir()
        .join(PACKAGE.ident())
}
