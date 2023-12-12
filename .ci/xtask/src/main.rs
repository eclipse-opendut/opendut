use clap::{Parser, Subcommand};
use tracing_subscriber::fmt::format::FmtSpan;
use crate::util::Package;

mod constants;
mod licenses;
mod util;

#[derive(Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    task: Task,
}
#[derive(Subcommand)]
enum Task {
    GenerateLicenses {
        #[arg()]
        package: Package,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .compact()
        .init();

    std::env::set_current_dir(constants::workspace_dir())?;

    let cli = Cli::parse();
    match cli.task {
        Task::GenerateLicenses { package } => licenses::generate_licenses(package),
    }
}
