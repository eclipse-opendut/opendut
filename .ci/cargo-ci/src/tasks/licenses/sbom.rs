use std::path::PathBuf;
use std::process::Command;
use tracing::info;
use crate::core::dependency::Crate;

use super::*;

use serde_spdx::spdx::v_2_3::{Spdx, SpdxItemPackages};


#[derive(Debug, clap::Parser)]
pub struct SbomCli;

#[tracing::instrument(skip_all)]
pub fn generate_sboms(packages: PackageSelection) -> crate::Result {
    util::install_crate(Crate::CargoSbom)?;

    for package in packages.iter() {
        generate_sbom(package)?
    }

    info!("Generated SBOMs in: {}", out_dir().display());
    Ok(())
}

pub fn generate_sbom(package: Package) -> crate::Result {
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
                clarify_license_information(package)
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

fn clarify_license_information(package: SpdxItemPackages) -> SpdxItemPackages {
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
                | "BSD-2-Clause OR Apache-2.0 OR MIT"
                | "BSD-2-Clause OR MIT OR Apache-2.0"
                | "CC0-1.0 OR MIT-0 OR Apache-2.0"
                | "MIT OR Apache-2.0"
                | "MIT OR Apache-2.0 OR BSD-1-Clause"
                | "MIT OR Apache-2.0 OR Zlib"
                | "MIT OR Zlib OR Apache-2.0"
                | "Zlib OR Apache-2.0 OR MIT"
                | "0BSD OR MIT OR Apache-2.0"
                => "Apache-2.0",

                "BSD-3-Clause OR MIT" => "BSD-3-Clause",
                "GPL-2.0 OR BSD-3-Clause" => "BSD-3-Clause",
                "ISC AND (Apache-2.0 OR ISC) AND OpenSSL" => "ISC AND OpenSSL",
                "ISC AND (Apache-2.0 OR ISC)" => "ISC",
                "MIT AND (MIT OR Apache-2.0)" => "MIT",
                "Unlicense OR MIT" => "MIT",
                "(Apache-2.0 OR MIT) AND BSD-3-Clause" => "Apache-2.0 AND BSD-3-Clause",
                "(MIT OR Apache-2.0) AND Unicode-3.0" => "Apache-2.0 AND Unicode-3.0",

                "Apache-2.0"
                | "Apache-2.0 WITH LLVM-exception"
                | "BSD-2-Clause"
                | "BSD-3-Clause"
                | "BSD-3-Clause AND MIT"
                | "BSL-1.0"
                | "CC0-1.0"
                | "ISC"
                | "MIT"
                | "MIT AND Apache-2.0"
                | "MIT AND BSD-3-Clause"
                | "MIT-0"
                | "MPL-2.0"
                | "PostgreSQL"
                | "Unicode-3.0"
                | "Zlib"
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
}

#[tracing::instrument(skip_all)]
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
