use clap::{ArgAction, Parser, Subcommand};
use dotenv::dotenv;

use crate::project::{boolean_env_var, check_dot_env_variables, OPENDUT_THEO_DISABLE_ENV_CHECKS};
use crate::project::make_dist::make_distribution_if_not_present;

mod project;
mod util;
mod docker;

#[derive(Debug, Parser)]
#[command(name = "opendut-theo")]
#[command(about = "opendut-theo - Test harness environment operator.")]
#[command(long_version = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Disable environment variable checks (OPENDUT_THEO_DISABLE_ENV_CHECKS=true)
    #[clap(long, short, action=ArgAction::SetFalse)]
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
    Edgar,
    #[command(about = "Show docker network.")]
    Network,
    #[command(about = "Destroy test environment.")]
    Destroy,
}


fn main() {
    dotenv().ok();
    let args = Cli::parse();
    if !args.disable_env_checks && !boolean_env_var(OPENDUT_THEO_DISABLE_ENV_CHECKS) {
        check_dot_env_variables();
    }

    match args.command {
        Commands::Build => {
            println!("Building testenv");
            make_distribution_if_not_present();
            docker::testenv::build::build_testenv();
        }
        Commands::Start => {
            make_distribution_if_not_present();
            println!("Starting testenv");
            docker::testenv::start::start_testenv();
        }
        Commands::Stop => {
            println!("Stopping testenv");
            docker::testenv::stop::stop_testenv();
        }
        Commands::Network => {
            docker::network::docker_inspect_network();
        }
        Commands::Destroy => {
            println!("Destroying testenv");
            docker::testenv::destroy::destroy_testenv();
        }
        Commands::Edgar => {
            docker::testenv::edgar::run_edgar();
        }
    }
}
