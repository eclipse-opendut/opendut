use std::path::PathBuf;
use cicero::path::repo_path;
use cicero::command_exit_ok::CommandExitOk;
use crate::fs;
use crate::core::commands;
use crate::constants;
use crate::core::types::parsing::package::PackageSelection;
use crate::Package;

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
    pub fn run(self, packages: PackageSelection) -> anyhow::Result<()> {
        match self.task {
            TaskCli::Check => {
                check::check_licenses()?;
            }
            TaskCli::Json => {
                for package in packages.iter() {
                    json::export_json(&package)?
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
    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn check_licenses() -> anyhow::Result<()> {
        commands::CARGO_DENY.command()
            .arg("check")
            .arg("--config").arg(cargo_deny_toml())
            .arg("--allow=no-license-field") //As of cargo-deny v0.18.9, there was no way to ignore this lint in the deny.toml, so we set it here. The lint gave warnings for some transitive dependencies, where we cannot immediately do anything about it. Cargo-deny should also fall back to reading the license from the license file in the source code of those crates, according to this: https://embarkstudios.github.io/cargo-deny/checks/licenses/diags.html#no-license-field
            .current_dir(repo_path!())
            .status_exit_ok()?;
        Ok(())
    }
}

pub mod json {
    use std::path::Path;
    use std::process::Stdio;
    use tracing::info;

    use super::*;

    #[tracing::instrument(skip_all)]
    pub fn export_json(package: &Package) -> anyhow::Result<()> {
        let out_file = out_file(package);
        fs::create_dir_all(out_file.parent().unwrap())?;

        export_json_with_out_path(package, &out_file)
    }

    #[tracing::instrument(skip_all)]
    pub fn export_json_with_out_path(package: &Package, out_file: &Path) -> anyhow::Result<()> {
        commands::CARGO_DENY.command()
            .arg("--exclude-dev")
            .arg("list")
            .arg("--config").arg(cargo_deny_toml())
            .arg("--layout=crate")
            .arg("--format=json")
            .current_dir(repo_path!().join(package.name))
            .stdout(Stdio::from(std::fs::File::create(out_file)?))
            .status_exit_ok()?;

        info!("Wrote licenses for package '{package}' to path: {}", out_file.display());

        Ok(())
    }

    pub fn out_file(package: &Package) -> PathBuf {
        constants::target_dir()
            .join("licenses")
            .join(out_file_name(package))
    }
    pub fn out_file_name(package: &Package) -> String {
        format!("{package}.licenses.json")
    }
}

mod texts {
    use super::*;
    use crate::fs;
    use std::path::PathBuf;
    use tracing::info;
    use crate::core::constants;

    #[derive(Debug, clap::Parser)]
    pub struct TextsCli;

    #[tracing::instrument(skip_all)]
    pub fn collect_license_texts() -> anyhow::Result<()> {
        let out_dir = out_dir();
        fs::create_dir_all(&out_dir)?;

        let out_path = out_dir.join("NOTICES.yaml");

        commands::CARGO_BUNDLE_LICENSES.command()
            .args(["--format=yaml", "--output", out_path.to_str().unwrap()])
            .status_exit_ok()?;

        info!("Generated bundle of license texts here: {}", out_path.display());

        Ok(())
    }

    pub fn out_dir() -> PathBuf {
        constants::target_dir().join("license-texts")
    }
}


fn cargo_deny_toml() -> PathBuf {
    repo_path!(".ci/cargo-deny.toml")
}
