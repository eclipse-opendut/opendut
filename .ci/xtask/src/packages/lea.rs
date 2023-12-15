use std::path::PathBuf;
use std::process::Command;

use crate::Package;
use crate::util::RunRequiringSuccess;

const PACKAGE: &Package = &Package::Lea;


#[derive(Debug, clap::Subcommand)]
pub enum LeaTask {
    Build,
    /// Start a development server for LEA which watches for file changes.
    Watch,
}
impl LeaTask {
    #[tracing::instrument]
    pub fn handle_task(self) -> anyhow::Result<()> {
        match self {
            LeaTask::Build => build::build_release()?,
            LeaTask::Watch => watch::watch()?,
        };
        Ok(())
    }
}

pub mod build {
    use super::*;

    #[tracing::instrument]
    pub fn build_release() -> anyhow::Result<()> {
        crate::util::install_crate("trunk")?;

        let working_dir = self_dir();
        let out_dir = out_dir();

        Command::new("trunk")
            .args([
                "build",
                "--release",
                "--dist", &out_dir.display().to_string(),
            ])
            .current_dir(working_dir)
            .run_requiring_success();

        Ok(())
    }
    pub fn out_dir() -> PathBuf {
        self_dir().join("dist")
    }
}

pub mod licenses {
    use super::*;

    pub fn generate_licenses() -> anyhow::Result<()> {
        crate::tasks::licenses::json::export_json(PACKAGE)
    }
    pub fn out_file() -> PathBuf {
        crate::tasks::licenses::json::out_file(PACKAGE)
    }
}

pub mod watch {
    use super::*;

    #[tracing::instrument]
    pub fn watch() -> anyhow::Result<()> {
        crate::util::install_crate("trunk")?;

        Command::new("trunk")
            .arg("watch")
            .current_dir(self_dir())
            .run_requiring_success();
        Ok(())
    }
}

pub fn self_dir() -> PathBuf {
    crate::constants::workspace_dir()
        .join(PACKAGE.ident())
}
