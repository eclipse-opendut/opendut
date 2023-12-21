use clap::{Parser, Subcommand};
use dotenv::dotenv;
use crate::project::check_dot_env_variables;
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
    check_dot_env_variables();
    let args = Cli::parse();

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
