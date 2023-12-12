use std::process::Command;

use anyhow::Result;


#[tracing::instrument]
pub fn install_crate(name: &str) -> Result<()> {
    Command::new("cargo")
        .arg("install")
        .arg(name)
        .status()?;
    Ok(())
}
