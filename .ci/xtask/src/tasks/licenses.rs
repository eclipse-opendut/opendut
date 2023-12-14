use std::fs;
use std::path::PathBuf;
use std::process::Command;

use clap::Subcommand;
use strum::IntoEnumIterator;

use crate::{constants, util};
use crate::Package;

#[derive(Debug, Subcommand)]
pub enum LicensesTask {
    /// Generate a license representation in JSON
    Json {
        #[arg(long)]
        package: Option<Package>,
    },
}
impl LicensesTask {
    #[tracing::instrument]
    pub fn handle_task(self) -> anyhow::Result<()> {
        match self {
            LicensesTask::Json { package } => match package {
                Some(package) => json::export_json(&package)?,
                None => {
                    for package in Package::iter() {
                        json::export_json(&package)?
                    }
                }
            }
        };
        Ok(())
    }
}


pub mod json {
    use super::*;

    #[tracing::instrument]
    pub fn export_json(package: &Package) -> anyhow::Result<()> {
        util::install_crate("cargo-deny")?;

        let out_file = out_file(package);
        fs::create_dir_all(out_file.parent().unwrap())?;

        Command::new("sh")
            .arg("-c")
            .arg(format!("cargo deny --exclude-dev list --layout crate --format json > {}", out_file.display()))
            .status()?;

        log::debug!("Wrote licenses for package '{package}' to path: {}", out_file.display());

        Ok(())
    }

    pub fn out_file(package: &Package) -> PathBuf {
        constants::target_dir()
            .join("licenses")
            .join(format!("{}.licenses.json", package.ident()))
    }
}
