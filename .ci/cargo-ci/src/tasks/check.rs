use std::process::Command;

use cicero::command_exit_ok::CommandExitOk;

use crate::tasks::licenses::check::check_licenses;
use crate::tasks::test;

/// Performs verification tasks.
#[derive(Debug, clap::Parser)]
pub struct CheckCli {
    /// activate all available features
    #[arg(long, default_value="true")]
    all_features: bool,
    /// specify features to activate
    #[arg(long)]
    features: Vec<String>,
}

impl CheckCli {
    pub fn run(self) -> anyhow::Result<()> {
        check(self.all_features, self.features)
    }
}

#[tracing::instrument(skip_all)]
pub fn check(all_features: bool, features: Vec<String>) -> anyhow::Result<()> {

    test::test(test::TestCli {
        all_features,
        features,
        disable_logging: true,
        test_name: None,
        pass_through: vec![],
    })?;

    clippy()?;

    check_licenses()?;

    Ok(())
}

#[tracing::instrument]
fn clippy() -> anyhow::Result<()> {
    Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-features",
        ])
        .status_exit_ok()?;
    Ok(())
}
