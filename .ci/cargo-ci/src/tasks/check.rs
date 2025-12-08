use std::process::Command;

use crate::tasks::licenses::check::check_licenses;
use crate::tasks::test;
use crate::util::RunRequiringSuccess;

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
    pub fn default_handling(self) -> crate::Result {
        check(self.all_features, self.features)
    }
}

#[tracing::instrument(skip_all)]
pub fn check(all_features: bool, features: Vec<String>) -> crate::Result {

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
fn clippy() -> crate::Result {
    Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-features",
        ])
        .run_requiring_success()
}
