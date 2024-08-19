use std::process::Command;

use crate::tasks::licenses::check::check_licenses;
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

#[tracing::instrument]
pub fn check(all_features: bool, features: Vec<String>) -> crate::Result {

    test(all_features, features)?;

    clippy()?;

    check_licenses()?;

    Ok(())
}

#[tracing::instrument]
fn test(all_features: bool, features: Vec<String>) -> crate::Result {

    let mut command = Command::new("cargo");

    command.arg("test");

    if features.is_empty() {
        if all_features {
            command.arg("--all-features");
        }
    }
    else {
        command.arg("--features");
        for feature in features {
            command.arg(feature);
        }
    }

    command.run_requiring_success()
}

#[tracing::instrument]
fn clippy() -> crate::Result {
    Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
        ])
        .run_requiring_success()
}
