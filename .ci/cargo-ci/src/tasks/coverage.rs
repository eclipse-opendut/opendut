use crate::fs;
use std::path::PathBuf;

use cicero::command_exit_ok::CommandExitOk;
use tracing::{debug, info, warn};
use crate::core::commands::CARGO_TARPAULIN;
use crate::core::constants;

/// Generate a unit test coverage report
#[derive(Debug, clap::Parser)]
pub struct CoverageCli;

impl CoverageCli {
    pub fn run(self) -> anyhow::Result<()> {
        coverage()
    }
}


#[tracing::instrument]
pub fn coverage() -> anyhow::Result<()> {
    clean()?;

    let out_dir = out_dir();
    fs::create_dir_all(&out_dir)?;

    CARGO_TARPAULIN.command()
        .args([
            "--all-features",
            "--workspace",
            "--timeout=1800", // 30 minutes timeout, because VIPER compiles code during tests, which can take a long time in the CI/CD runner
            "--out", "xml", "html", "lcov",
            "--output-dir", out_dir.to_str().unwrap(),
            "--fail-immediately",
        ])
        .status_exit_ok()?;

    let files = fs::read_dir(&out_dir)?
        .filter_map(|entry| {
            entry
                .inspect_err(|source| warn!("Ignoring coverage file which could not be read: {source}"))
                .ok()
        })
        .filter(|entry| entry.path().is_file());

    for file in files {
        let file_name = file.file_name().into_string().unwrap();
        fs::rename(file.path(), out_dir.join(format!("coverage.{file_name}")))?;
    }

    info!("Placed coverage files into: {}", out_dir.display());

    Ok(())
}

#[tracing::instrument]
pub fn clean() -> anyhow::Result<()> {
    let out_dir = out_dir();
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
        debug!("Cleaned coverage output directory at: {out_dir:?}");
    }
    Ok(())
}

pub fn out_dir() -> PathBuf {
    constants::target_dir().join("coverage")
}
