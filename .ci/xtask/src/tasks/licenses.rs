use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::{constants, util};
use crate::Package;


#[tracing::instrument]
pub fn generate_licenses(package: &Package) -> anyhow::Result<()> {
    util::install_crate("cargo-deny")?;

    let out_file = out_file(package);
    fs::create_dir_all(&out_file.parent().unwrap())?;

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
