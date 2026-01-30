use std::process::ExitCode;
use clap::Parser;

use opendut_model::peer::PeerId;

#[derive(Parser)]
#[command(name = "opendut-edgar")]
#[command(about = "Connect your ECU to openDuT.")]
#[command(long_version = crate::FORMATTED_VERSION)]
enum Commands {
    /// Launches the EDGAR Service
    Service {
        /// Start with the provided ID
        #[arg(long)]
        id: Option<PeerId>,
    },
    Setup(crate::setup::cli::SetupCli),
}


pub async fn cli() -> anyhow::Result<ExitCode> {
    match Commands::parse() {
        Commands::Service { id } => {
            crate::service::start::launch(id).await?;

            Ok(ExitCode::SUCCESS)
        },
        Commands::Setup(cli) => cli.run().await,
    }
}
