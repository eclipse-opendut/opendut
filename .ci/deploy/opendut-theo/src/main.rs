use crate::commands::dev::DevCli;
use crate::commands::testenv::TestenvCli;
use crate::commands::vagrant::VagrantCli;
use crate::core::project::ProjectRootDir;
use crate::core::project;
use clap::Parser;
use std::path::PathBuf;

mod core;
mod commands;

#[derive(Parser)]
#[command(name = "opendut-theo")]
#[command(about = "opendut-theo - Test harness environment operator.")]
#[command(long_version = None)]
enum Cli {
    Testenv(TestenvCli),
    Vagrant(VagrantCli),
    Dev(DevCli),
}


fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr)
        .init();

    PathBuf::project_dir_verify();
    project::load_environment_variables_from_dot_env_file();

    match Cli::parse() {
        Cli::Testenv(cli) => cli.run()?,
        Cli::Vagrant(cli) => cli.run()?,
        Cli::Dev(cli) => cli.run()?,
    };
    Ok(())
}
