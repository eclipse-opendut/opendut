use cicero::command_exit_ok::CommandExitOk;
use anyhow::Context;
use std::process::Command;

/// Start the application
#[derive(Clone, clap::Args)]
pub struct RunCli {
    /// Space or comma separated list of features to activate
    #[arg(long)]
    pub features: Vec<String>,

    /// Additional parameters to pass through to the started program
    #[arg(raw=true)]
    pub passthrough: Vec<String>,
}

impl RunCli {
    #[tracing::instrument(name="run", skip(self))]
    pub fn run(&self, package: &crate::Package) -> anyhow::Result<()> {
        self.command(package).status_exit_ok()?;

        Ok(())
    }

    #[tracing::instrument(name="run", skip(self))]
    pub async fn run_async(&self, package: &crate::Package) -> anyhow::Result<()> {
        let status = tokio::process::Command::from(self.command(package))
            .kill_on_drop(true)
            .status().await
            .with_context(|| format!("Failed to run package '{}'", package.name))?;

        anyhow::ensure!(status.success(), "Error while running package '{}': {status}", package.name);

        Ok(())
    }

    fn command(&self, package: &crate::Package) -> Command {
        let mut command = Command::new("cargo");
        command
            .args(["run", "--package", package.name])
            .arg("--features").arg(self.features.join(","))
            .arg("--")
            .args(&self.passthrough);
        
        command
    }
}
