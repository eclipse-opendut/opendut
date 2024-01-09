use std::process::Command;
use crate::tasks::licenses::check::check_licenses;

use crate::util::RunRequiringSuccess;

/// Performs verification tasks.
#[derive(Debug, clap::Parser)]
pub struct CheckCli {

}

impl CheckCli {
    pub fn default_handling(self) -> crate::Result {
        check()
    }
}

#[tracing::instrument]
pub fn check() -> crate::Result {

    test();

    clippy();

    check_licenses()?;

    Ok(())
}

#[tracing::instrument]
fn test() {
    Command::new("cargo")
        .args([
            "test",
            "--all-features",
        ])
        .run_requiring_success();
}

#[tracing::instrument]
fn clippy() {
    Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--exclude=opendut-lea" // rustc (1.75.0) crashes for opendut-lea.
        ])
        .run_requiring_success();
}
