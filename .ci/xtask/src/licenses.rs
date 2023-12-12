use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::{constants, util};
use crate::util::Package;


#[tracing::instrument]
pub fn generate_licenses(package: Package) -> anyhow::Result<()> {
    util::install_crate("cargo-deny")?;

    let licenses_dir = licenses_dir();
    fs::create_dir_all(&licenses_dir)?;

    let target = licenses_dir.join(format!("{package}.licenses.json"));

    Command::new("sh")
        .arg("-c")
        .arg(format!("cargo deny --exclude-dev list --layout crate --format json > {}", target.display()))
        .status()?;

    log::debug!("Wrote licenses for package '{package}' to path: {}", target.display());

    Ok(())
}

fn licenses_dir() -> PathBuf {
    constants::ci_dir().join("licenses")
}
