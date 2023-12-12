use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::{constants, util};
use crate::Package;


#[tracing::instrument]
pub fn generate_licenses(package: &Package) -> anyhow::Result<PathBuf> {
    util::install_crate("cargo-deny")?;

    let target = licenses_file(package);
    fs::create_dir_all(&target.parent().unwrap())?;

    Command::new("sh")
        .arg("-c")
        .arg(format!("cargo deny --exclude-dev list --layout crate --format json > {}", target.display()))
        .status()?;

    log::debug!("Wrote licenses for package '{package}' to path: {}", target.display());

    Ok(target)
}

fn licenses_file(package: &Package) -> PathBuf {
    constants::target_dir().join("licenses").join(format!("{}.licenses.json", package.ident()))
}
