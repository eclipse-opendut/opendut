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
    pub fn default_handling(&self, package: crate::Package) -> crate::Result {
        Command::new("cargo")
            .args(["run", "--package", &package.ident()])
            .arg("--features").arg(self.features.join(","))
            .arg("--")
            .args(&self.passthrough)
            .status()?;

        Ok(())
    }
}
