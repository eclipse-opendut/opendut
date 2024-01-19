use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::core::Result;
use crate::commands::dev::DevCli;
use crate::commands::testenv::TestenvCli;
use crate::commands::vagrant::VagrantCli;
use crate::core::project::{dot_env_create_defaults, ProjectRootDir};

mod core;
mod commands;

#[derive(Debug, Parser)]
#[command(name = "opendut-theo")]
#[command(about = "opendut-theo - Test harness environment operator.")]
#[command(long_version = None)]
struct Cli {
    #[command(subcommand)]
    command: TaskCli,
}

#[derive(Debug, Subcommand)]
enum TaskCli {
    Testenv(TestenvCli),
    Vagrant(VagrantCli),
    Dev(DevCli),
    #[command(about = "Show netbird application versions.")]
    NetbirdVersions,
}


fn main() -> Result {
    let args = Cli::parse();
    PathBuf::project_dir_verify();

    // load environment variables from .env file
    dot_env_create_defaults();

    match args.command {
        TaskCli::Testenv(implementation) => { implementation.default_handling()? }
        TaskCli::NetbirdVersions => {
            let metadata = core::metadata::cargo_netbird_versions();
            println!("Versions: {:?}", metadata);
        }
        TaskCli::Vagrant(implementation) => { implementation.default_handling()? }
        TaskCli::Dev(implementation) => { implementation.default_handling()? }
    }
    Ok(())
}
