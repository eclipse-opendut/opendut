use std::path::PathBuf;
use clap::{Parser, Subcommand};

use crate::commands::testenv::TestenvCli;
use crate::commands::vagrant::VagrantCli;
use crate::core::project::ProjectRootDir;

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
    #[command(about = "Show netbird application versions.")]
    NetbirdVersions,
}


fn main() {
    let args = Cli::parse();
    PathBuf::project_dir_verify();

    match args.command {
        TaskCli::Testenv(implementation) => { implementation.default_handling() }
        TaskCli::NetbirdVersions => {
            let metadata = core::metadata::cargo_netbird_versions();
            println!("Versions: {:?}", metadata);
        }
        TaskCli::Vagrant(implementation) => { implementation.default_handling() }
    }
}
