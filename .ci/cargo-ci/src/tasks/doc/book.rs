use cicero::{command_exit_ok::CommandExitOk, path::repo_path};
use crate::core::commands::MDBOOK;
use super::*;

#[tracing::instrument]
pub fn open() -> anyhow::Result<()> {
    MDBOOK.command()
        .arg("serve")
        .arg("--open")
        .arg("--port=4000")
        .arg("--dest-dir").arg(out_dir())
        .current_dir(doc_dir())
        .status_exit_ok()?;
    Ok(())
}

#[tracing::instrument]
pub fn build() -> anyhow::Result<()> {
    let out_dir = out_dir();

    MDBOOK.command()
        .arg("build")
        .arg("--dest-dir").arg(&out_dir)
        .current_dir(doc_dir())
        .status_exit_ok()?;

    Ok(())
}

fn doc_dir() -> PathBuf {
    repo_path!("doc/")
}

pub fn out_dir() -> PathBuf {
    crate::constants::target_dir().join("book")
}
