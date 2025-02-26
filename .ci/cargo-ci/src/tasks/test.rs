use std::process::Command;
use crate::core::util::RunRequiringSuccess;

/// Run tests with better defaults (all features, logging and nocapture enabled)
#[derive(Debug, clap::Parser)]
pub struct TestCli {
    /// Activate all available features
    #[arg(long, default_value="true")]
    pub all_features: bool,
    /// Specify features to activate
    #[arg(long)]
    pub features: Vec<String>,
    /// Disable printing of logs
    #[arg(long)]
    pub disable_logging: bool,
    /// Which test to run
    #[arg()]
    pub test_name: Option<String>,
    /// Additional parameters to pass through to the started program
    #[arg(raw=true)]
    pub pass_through: Vec<String>,
}
impl TestCli {
    pub fn default_handling(self) -> crate::Result {
        test(self)
    }
}

#[tracing::instrument(skip_all)]
pub fn test(params: TestCli) -> crate::Result {
    let TestCli { all_features, features, disable_logging, test_name, pass_through } = params;

    let mut command = Command::new("cargo");
    command.arg("test");

    if disable_logging {
        command.env("RUST_LOG", "error");
    } else {
        command.env("RUST_LOG", "info,opendut=trace");
    }

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

    if let Some(test_name) = test_name {
        command.arg(test_name);
    }
    
    command.args(pass_through);

    command.args(["--", "--nocapture"]);

    command.run_requiring_success()
}
