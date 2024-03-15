use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::process::Command;

use crate::{constants, util};
use crate::core::types::parsing::package::PackageSelection;
use crate::Package;
use crate::util::RunRequiringSuccess;

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
    #[tracing::instrument]
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

    #[tracing::instrument]
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

    use crate::core::dependency::Crate;

    use super::*;

    #[tracing::instrument]
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
            .stdout(Stdio::from(File::create(&out_file)?))
            .run_requiring_success()?;

        log::info!("Wrote licenses for package '{package}' to path: {}", out_file.display());

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

mod sbom {
    use crate::core::dependency::Crate;

    use super::*;

    #[derive(Debug, clap::Parser)]
    pub struct SbomCli;

    #[tracing::instrument]
    pub fn generate_sboms(packages: PackageSelection) -> crate::Result {
        util::install_crate(Crate::CargoSbom)?;

        for package in packages.iter() {
            generate_sbom(package)?
        }

        log::info!("Generated SBOMs in: {}", out_dir().display());
        Ok(())
    }

    pub fn generate_sbom(package: Package) -> crate::Result {
        use serde_spdx::spdx::v_2_3::{Spdx, SpdxItemPackages};

        let sbom_dir = out_dir();
        fs::create_dir_all(&sbom_dir)?;

        let sbom = Command::new("cargo-sbom")
            .args(["--cargo-package", &package.ident(), "--output-format", "spdx_json_2_3"])
            .output()?
            .stdout;
        let sbom = std::str::from_utf8(&sbom)?;

        let mut sbom: Spdx = serde_json::from_str(sbom)?;

        { //remove erronous file information added by cargo-sbom (was two entries for the respective package's binary, without additional information)
            sbom.files = None;
        }

        { //override license information for crates with unclear license
            sbom.packages = sbom.packages.map(|packages|
                packages.into_iter().map(|package| {
                    if package.name == "ring" {
                        SpdxItemPackages {
                            license_concluded: Some(String::from("MIT AND ISC AND OpenSSL")), //comply with all licenses to be on the safe side
                            license_declared: Some(String::from("NOASSERTION")),
                            ..package
                        }
                    } else {
                        let license_concluded = package.license_concluded.as_ref().map(|license|
                            // When selecting a license, choose Apache-2.0 where possible.
                            // Otherwise, select the most permissive license.

                            match license.as_ref() {
                                "Apache-2.0 OR MIT"
                                | "Apache-2.0 OR Apache-2.0 OR MIT"
                                | "Apache-2.0 OR BSL-1.0"
                                | "Apache-2.0 OR ISC OR MIT"
                                | "MIT OR Apache-2.0"
                                | "MIT OR Apache-2.0 OR BSD-1-Clause"
                                | "MIT OR Apache-2.0 OR Zlib"
                                | "MIT OR Zlib OR Apache-2.0"
                                | "Zlib OR Apache-2.0 OR MIT"
                                | "0BSD OR MIT OR Apache-2.0"
                                => "Apache-2.0",

                                "BSD-3-Clause OR MIT" => "BSD-3-Clause",
                                "Unlicense OR MIT" => "MIT",
                                "(Apache-2.0 OR MIT) AND BSD-3-Clause" => "Apache-2.0 AND BSD-3-Clause",
                                "(MIT OR Apache-2.0) AND Unicode-DFS-2016" => "Apache-2.0 AND Unicode-DFS-2016",

                                "Apache-2.0"
                                | "BSD-3-Clause"
                                | "BSL-1.0"
                                | "ISC"
                                | "MIT"
                                | "MIT AND Apache-2.0"
                                | "MIT AND BSD-3-Clause"
                                | "MIT-0"
                                | "Zlib"
                                | "MPL-2.0"
                                => license, //leave unchanged

                                other => panic!("Unknown license '{}' for package '{}'. Please define a mapping.", other, package.name)
                            }.to_string()
                        );

                        let license_declared = package.license_declared.as_ref().map(|license|
                            // Change slashes into "OR" to improve compatibility with external systems.

                            if license.contains('/') {

                                match license.as_ref() {
                                    "Apache-2.0/MIT"
                                    | "MIT/Apache-2.0"
                                    | "MIT / Apache-2.0"
                                    | "Apache-2.0 / MIT"
                                    => "Apache-2.0 OR MIT",

                                    "BSD-3-Clause/MIT"
                                    | "MIT/BSD-3-Clause"
                                    | "MIT / BSD-3-Clause"
                                    | "BSD-3-Clause / MIT"
                                    => "BSD-3-Clause OR MIT",

                                    "Unlicense/MIT"
                                    | "MIT/Unlicense"
                                    | "MIT / Unlicense"
                                    | "Unlicense / MIT"
                                    => "MIT OR Unlicense",

                                    "Apache-2.0/ISC/MIT"
                                    | "Apache-2.0 / ISC / MIT"
                                    | "Apache-2.0/MIT/ISC"
                                    | "Apache-2.0 / MIT / ISC"
                                    | "ISC/Apache-2.0/MIT"
                                    | "ISC / Apache-2.0 / MIT"
                                    | "ISC/MIT/Apache-2.0"
                                    | "ISC / MIT / Apache-2.0"
                                    | "MIT/ISC/Apache-2.0"
                                    | "MIT / Apache-2.0 / ISC"
                                    | "MIT/Apache-2.0/ISC"
                                    | "MIT / ISC / Apache-2.0"
                                    => "MIT OR ISC OR Apache-2.0",

                                    other => panic!("Unmatched license specification '{}' for package '{}'. Please check mapping.", other, package.name)
                                }.to_string()
                            }
                            else {
                                license.to_string()
                            }
                        );
                        SpdxItemPackages {
                            license_concluded,
                            license_declared,
                            ..package
                        }
                    }
                }).collect::<Vec<_>>()
            );
        }

        let sbom = serde_json::to_string_pretty(&sbom)?;

        fs::write(
            sbom_dir.join(format!("{}-sbom.spdx.json", package.ident())),
            sbom
        )?;

        Ok(())
    }

    #[tracing::instrument]
    fn clean() -> crate::Result {
        let sbom_dir = out_dir();
        if sbom_dir.exists() {
            fs::remove_dir_all(sbom_dir)?;
        }
        Ok(())
    }

    pub fn out_dir() -> PathBuf {
        constants::target_dir().join("sbom")
    }
}

mod texts {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use crate::core::{constants, util};
    use crate::core::dependency::Crate;
    use crate::core::util::RunRequiringSuccess;

    #[derive(Debug, clap::Parser)]
    pub struct TextsCli;

    #[tracing::instrument]
    pub fn collect_license_texts() -> crate::Result {
        util::install_crate(Crate::CargoBundleLicenses)?;

        let out_dir = out_dir();
        fs::create_dir_all(&out_dir)?;

        let out_path = out_dir.join("NOTICES.yaml");

        Command::new("cargo-bundle-licenses")
            .args(["--format=yaml", "--output", out_path.to_str().unwrap()])
            .run_requiring_success()?;

        log::info!("Generated bundle of license texts here: {}", out_path.display());

        Ok(())
    }

    pub fn out_dir() -> PathBuf {
        constants::target_dir().join("license-texts")
    }
}


fn cargo_deny_toml() -> PathBuf {
    constants::ci_dir().join("cargo-deny.toml")
}
