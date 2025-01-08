use crate::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::{constants, util};
use crate::core::types::parsing::package::PackageSelection;
use crate::Package;
use crate::util::RunRequiringSuccess;

mod sbom;


/// Check or export licenses
#[derive(Debug, clap::Parser)]
pub struct LicensesCli {
    #[command(subcommand)]
    pub task: TaskCli,
}

#[derive(Debug, clap::Subcommand)]
pub enum TaskCli {
    /// Check for license violations and security vulnerabilities
    Check,
    /// Generate a license report in JSON format
    Json,
    /// Generate a license report in SBOM format
    Sbom,
    /// Collect the license texts
    Texts,
}

impl LicensesCli {
    #[tracing::instrument(skip_all)]
    pub fn default_handling(self, packages: PackageSelection) -> crate::Result {
        match self.task {
            TaskCli::Check => {
                check::check_licenses()?;
            }
            TaskCli::Json => {
                for package in packages.iter() {
                    json::export_json(package)?
                }
            }
            TaskCli::Sbom => {
                sbom::generate_sboms(packages)?
            }
            TaskCli::Texts => {
                texts::collect_license_texts()?
            }
        };
        Ok(())
    }
}

pub mod check {
    use crate::core::dependency::Crate;

    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn check_licenses() -> crate::Result {
        util::install_crate(Crate::CargoDeny)?;

        Command::new("cargo-deny")
            .arg("check")
            .arg("--config").arg(cargo_deny_toml())
            .run_requiring_success()
    }
}

pub mod json {
    use std::process::Stdio;
    use tracing::info;

    use crate::core::dependency::Crate;

    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn export_json(package: Package) -> crate::Result {
        util::install_crate(Crate::CargoDeny)?;

        let out_file = out_file(package);
        fs::create_dir_all(out_file.parent().unwrap())?;

        Command::new("cargo-deny")
            .arg("--exclude-dev")
            .arg("list")
            .arg("--config").arg(cargo_deny_toml())
            .arg("--layout=crate")
            .arg("--format=json")
            .stdout(Stdio::from(std::fs::File::create(&out_file)?))
            .run_requiring_success()?;

        info!("Wrote licenses for package '{package}' to path: {}", out_file.display());

        Ok(())
    }

    pub fn out_file(package: Package) -> PathBuf {
        constants::target_dir()
            .join("licenses")
            .join(out_file_name(package))
    }
    pub fn out_file_name(package: Package) -> String {
        format!("{}.licenses.json", package.ident())
    }
}

mod texts {
    use crate::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use tracing::info;
    use crate::core::{constants, util};
    use crate::core::dependency::Crate;
    use crate::core::util::RunRequiringSuccess;

    #[derive(Debug, clap::Parser)]
    pub struct TextsCli;

    #[tracing::instrument(skip_all)]
    pub fn collect_license_texts() -> crate::Result {
        util::install_crate(Crate::CargoBundleLicenses)?;

        let out_dir = out_dir();
        fs::create_dir_all(&out_dir)?;

        let out_path = out_dir.join("NOTICES.yaml");

        Command::new("cargo-bundle-licenses")
            .args(["--format=yaml", "--output", out_path.to_str().unwrap()])
            .run_requiring_success()?;

        info!("Generated bundle of license texts here: {}", out_path.display());

        Ok(())
    }

    pub fn out_dir() -> PathBuf {
        constants::target_dir().join("license-texts")
    }
}


fn cargo_deny_toml() -> PathBuf {
    constants::ci_code_dir().join("cargo-deny.toml")
}
