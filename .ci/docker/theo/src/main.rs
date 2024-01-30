use std::path::PathBuf;
use clap::{Parser, Subcommand};
use commands::inside_edgar_container;
use commands::inside_edgar_container::InsideEdgarContainerCli;

use crate::commands::dev::DevCli;
use crate::commands::testenv::TestenvCli;
use crate::commands::vagrant::VagrantCli;
use crate::core::{project, Result};
use crate::core::project::ProjectRootDir;

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

    InsideEdgarContainer(InsideEdgarContainerCli),
}


fn main() -> Result {
    let args = Cli::parse();

    match args.command {
        TaskCli::InsideEdgarContainer(_) => {
            /* Don't set up repo-derived variables, since we don't have the repo mounted into the containers. */
        },
        _ => {
            PathBuf::project_dir_verify();
            project::load_environment_variables_from_dot_env_file();
        }
    };

    match args.command {
        TaskCli::Testenv(implementation) => { implementation.default_handling()? }
        TaskCli::Vagrant(implementation) => { implementation.default_handling()? }
        TaskCli::Dev(implementation) => { implementation.default_handling()? }
        TaskCli::NetbirdVersions => {
            let metadata = core::metadata::cargo_netbird_versions();
            println!("Versions: {:?}", metadata);
        }

        TaskCli::InsideEdgarContainer(cli) => {
            inside_edgar_container::call_managed_sh(cli.task)?;
        }
    };
    Ok(())
}
