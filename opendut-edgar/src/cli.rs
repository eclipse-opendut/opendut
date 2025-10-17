use clap::{Parser, Subcommand};
use uuid::Uuid;

use opendut_model::peer::PeerId;


#[derive(Parser)]
#[command(name = "opendut-edgar")]
#[command(about = "Connect your ECU to openDuT.")]
#[command(long_version = crate::FORMATTED_VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launches the EDGAR Service
    Service {
        /// Start with the provided ID
        #[arg(long)]
        id: Option<Uuid>,
    },
    Setup(crate::setup::cli::SetupCli),
}


pub async fn cli() -> anyhow::Result<()> {

    let args = Cli::parse();

    match args.command {
        Commands::Service { id } => {
            let id_override = id.map(PeerId::from);
            crate::service::start::launch(
                id_override,
            ).await
        },
        Commands::Setup(cli) => cli.run().await,
    }
}
