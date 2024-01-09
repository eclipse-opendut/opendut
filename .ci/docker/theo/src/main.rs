use std::path::PathBuf;
use clap::{ArgAction, Parser, Subcommand};
use dotenvy::dotenv;

use crate::commands::edgar::TestEdgarCli;
use crate::core::dist::make_distribution_if_not_present;
use crate::core::OPENDUT_THEO_DISABLE_ENV_CHECKS;
use crate::core::project::{boolean_env_var, check_dot_env_variables, ProjectRootDir};

mod core;
mod commands;

#[derive(Debug, Parser)]
#[command(name = "opendut-theo")]
#[command(about = "opendut-theo - Test harness environment operator.")]
#[command(long_version = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Disable environment variable checks (OPENDUT_THEO_DISABLE_ENV_CHECKS=true)
    #[clap(long, short, action=ArgAction::SetTrue)]
    disable_env_checks: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Build docker containers.")]
    Build,
    #[command(about = "Start test environment.")]
    Start,
    #[command(about = "Stop test environment.")]
    Stop,
    #[command(about = "Run edgar cluster creation.")]
    Edgar(TestEdgarCli),
    #[command(about = "Show docker network.")]
    Network,
    #[command(about = "Destroy test environment.")]
    Destroy,
    #[command(about = "Show netbird application versions.")]
    NetbirdVersions,
}


fn main() {
    // load environment variables from .env file
    dotenv().expect(".env file not found");
    let custom_env = PathBuf::project_path_buf().join(".env-theo");
    dotenvy::from_path(custom_env).expect(".env-theo file not found");

    let args = Cli::parse();
    if !args.disable_env_checks && !boolean_env_var(OPENDUT_THEO_DISABLE_ENV_CHECKS) {
        check_dot_env_variables();
    } else {
        println!("Skipping environment variable checks.");
    }

    match args.command {
        Commands::Build => {
            println!("Building testenv");
            make_distribution_if_not_present();
            commands::testenv::build::build_testenv();
        }
        Commands::Start => {
            make_distribution_if_not_present();
            println!("Starting testenv");
            commands::testenv::start::start_testenv();
        }
        Commands::Stop => {
            println!("Stopping testenv");
            commands::testenv::stop::stop_testenv();
        }
        Commands::Network => {
            commands::network::docker_inspect_network();
        }
        Commands::Destroy => {
            println!("Destroying testenv");
            commands::testenv::destroy::destroy_testenv();
        }
        Commands::Edgar(implementation) => implementation.default_handling(),

        Commands::NetbirdVersions => {
            core::metadata::cargo_netbird_versions();
        }
    }
}
