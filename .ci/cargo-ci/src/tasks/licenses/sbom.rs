use std::path::PathBuf;
use tracing::info;

use crate::core::commands;

use super::*;

use serde_spdx::spdx::v_2_3::{Spdx, SpdxItemPackages};


#[derive(Debug, clap::Parser)]
pub struct SbomCli;

#[tracing::instrument(skip_all)]
pub fn generate_sboms(packages: PackageSelection) -> crate::Result {
    for package in packages.iter() {
        generate_sbom(package)?
    }

    info!("Generated SBOMs in: {}", out_dir().display());
    Ok(())
}

pub fn generate_sbom(package: Package) -> crate::Result {
    let sbom_dir = out_dir();
    fs::create_dir_all(&sbom_dir)?;

    let sbom = commands::CARGO_SBOM.command()
        .args(["--cargo-package", &package.ident(), "--output-format", "spdx_json_2_3"])
        .output()?
        .stdout;
    let sbom = std::str::from_utf8(&sbom)?;

    let mut sbom: Spdx = serde_json::from_str(sbom)?;

    { //remove erronous file information added by cargo-sbom (was two entries for the respective package's binary, without additional information)
        sbom.files = None;
    }

    sbom.packages = sbom.packages.map(|spdx_packages| {
        let mut spdx_packages = spdx_packages.into_iter().map(|spdx_package| {
            clarify_license_information(spdx_package)
        }).collect::<Vec<_>>();

        let cargo_metadata = crate::metadata::cargo();
        spdx_packages.push(netbird_spdx_package(&cargo_metadata));

        if package == Package::Edgar {
            spdx_packages.push(cannelloni_spdx_package(&cargo_metadata));
            spdx_packages.push(rperf_spdx_package(&cargo_metadata));
        }

        spdx_packages
    });

    let sbom = serde_json::to_string_pretty(&sbom)?;

    fs::write(
        sbom_dir.join(format!("{}-sbom.spdx.json", package.ident())),
        sbom
    )?;

    Ok(())
}

/// Override license information for crates with unclear license.
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
                | "MIT OR Apache-2.0 OR LGPL-2.1-or-later"
                | "MIT OR Apache-2.0 OR Zlib"
                | "MIT OR MIT AND Apache-2.0"
                | "MIT OR Zlib OR Apache-2.0"
                | "MIT-0 OR MIT OR Apache-2.0"
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

fn netbird_spdx_package(cargo_metadata: &cargo_metadata::Metadata) -> SpdxItemPackages {

    let version = cargo_metadata.workspace_metadata["ci"]["netbird"]["version"].as_str()
        .expect("NetBird version not defined.");

    SpdxItemPackages {
        spdxid: format!("SPDXRef-Package-netbird-{version}"),
        comment: Some(String::from("Used as external software for coordinating VPN management.")),
        download_location: String::from("https://netbird.io/"),
        homepage: Some(String::from("https://netbird.io/")),
        license_concluded: Some(String::from("BSD-3-Clause")),
        license_declared: Some(String::from("BSD-3-Clause")),
        name: String::from("NetBird"),
        version_info: Some(String::from(version)),

        annotations: None,
        attribution_texts: None,
        built_date: None,
        checksums: None,
        copyright_text: None,
        description: None,
        external_refs: None,
        files_analyzed: None,
        has_files: None,
        license_comments: None,
        license_info_from_files: None,
        originator: None,
        package_file_name: None,
        package_verification_code: None,
        primary_package_purpose: None,
        release_date: None,
        source_info: None,
        summary: None,
        supplier: None,
        valid_until_date: None,
    }
}

fn cannelloni_spdx_package(cargo_metadata: &cargo_metadata::Metadata) -> SpdxItemPackages {

    let version = cargo_metadata.workspace_metadata["ci"]["cannelloni"]["version"].as_str()
        .expect("Cannelloni version not defined.");

    SpdxItemPackages {
        spdxid: format!("SPDXRef-Package-cannelloni-{version}"),
        comment: Some(String::from("Used as external software for forwarding CAN communication over HTTP. No direct linking takes place.")),
        download_location: String::from("https://github.com/mguentner/cannelloni"),
        homepage: Some(String::from("https://github.com/mguentner/cannelloni")),
        license_concluded: Some(String::from("GPL-2.0")),
        license_declared: Some(String::from("GPL-2.0")),
        name: String::from("Cannelloni"),
        version_info: Some(String::from(version)),

        annotations: None,
        attribution_texts: None,
        built_date: None,
        checksums: None,
        copyright_text: None,
        description: None,
        external_refs: None,
        files_analyzed: None,
        has_files: None,
        license_comments: None,
        license_info_from_files: None,
        originator: None,
        package_file_name: None,
        package_verification_code: None,
        primary_package_purpose: None,
        release_date: None,
        source_info: None,
        summary: None,
        supplier: None,
        valid_until_date: None,
    }
}

fn rperf_spdx_package(cargo_metadata: &cargo_metadata::Metadata) -> SpdxItemPackages {

    let version = cargo_metadata.workspace_metadata["ci"]["rperf"]["version"].as_str()
        .expect("Rperf version not defined.");

    SpdxItemPackages {
        spdxid: format!("SPDXRef-Package-rperf-{version}"),
        comment: Some(String::from("Used as external software for monitoring network performance. No direct linking takes place.")),
        download_location: String::from("https://github.com/opensource-3d-p/rperf"),
        homepage: Some(String::from("https://github.com/opensource-3d-p/rperf")),
        license_concluded: Some(String::from("GPL-3.0")),
        license_declared: Some(String::from("GPL-3.0")),
        name: String::from("Rperf"),
        version_info: Some(String::from(version)),

        annotations: None,
        attribution_texts: None,
        built_date: None,
        checksums: None,
        copyright_text: None,
        description: None,
        external_refs: None,
        files_analyzed: None,
        has_files: None,
        license_comments: None,
        license_info_from_files: None,
        originator: None,
        package_file_name: None,
        package_verification_code: None,
        primary_package_purpose: None,
        release_date: None,
        source_info: None,
        summary: None,
        supplier: None,
        valid_until_date: None,
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
