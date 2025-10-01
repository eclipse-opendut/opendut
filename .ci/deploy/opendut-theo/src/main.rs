use crate::commands::dev::DevCli;
use crate::commands::testenv::TestenvCli;
use crate::commands::vagrant::VagrantCli;
use crate::core::project::ProjectRootDir;
use crate::core::{project, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod core;
mod commands;

#[derive(Parser)]
#[command(name = "opendut-theo")]
#[command(about = "opendut-theo - Test harness environment operator.")]
#[command(long_version = None)]
struct Cli {
    #[command(subcommand)]
    command: TaskCli,
}

#[derive(Subcommand)]
enum TaskCli {
    Testenv(TestenvCli),
    Vagrant(VagrantCli),
    Dev(DevCli),
    /// Show NetBird application versions.
    NetbirdVersions,
}


fn main() -> Result {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr)
        .init();

    let args = Cli::parse();

    PathBuf::project_dir_verify();
    project::load_environment_variables_from_dot_env_file();

    match args.command {
        TaskCli::Testenv(implementation) => { implementation.default_handling()? }
        TaskCli::Vagrant(implementation) => { implementation.default_handling()? }
        TaskCli::Dev(implementation) => { implementation.default_handling()? }
        TaskCli::NetbirdVersions => {
            let metadata = core::metadata::cargo_netbird_versions();
            println!("Versions: {metadata:?}");
        }
    };
    Ok(())
}
