use clap::Parser;

pub use fs_err as fs;
use tracing_subscriber::EnvFilter;
pub(crate) use core::constants;
pub(crate) use core::metadata;
pub(crate) use core::types::Package;
use std::time::SystemTime;
use crate::core::types::parsing::package::PackageSelection;

mod core;
pub mod packages;
mod tasks;

shadow_rs::shadow!(build);
cicero::workspace!();


#[derive(clap::Parser)]
enum Cli {
    Check(tasks::check::CheckCli),
    Coverage(tasks::coverage::CoverageCli),
    Distribution(tasks::distribution::DistributionCliWithFilter),
    Doc(tasks::doc::DocCli),
    Licenses(tasks::licenses::LicensesCli),
    Test(tasks::test::TestCli),
    IntegrationTest(tasks::integration_tests::IntegrationTestCli),

    Carl(packages::carl::CarlCli),
    Cleo(packages::cleo::CleoCli),
    Edgar(packages::edgar::EdgarCli),
    Lea(packages::lea::LeaCli),
    Theo(packages::theo::TheoCli),
}

fn main() -> anyhow::Result<()> {
    cicero::init::tracing()
        .with_env_filter(
            EnvFilter::builder()
                .parse("info,opendut=trace,cicero=trace")?
        )
        .init();

    // Workaround for Cicero invalidating the build cache
    // when it is copied into a different CI step,
    // which we do for building the distribution in parts.
    if option_env!("CI").is_some() {
        let executable = std::env::current_exe()?;
        if executable.exists() {
            fs::File::open(executable)?
                .set_modified(SystemTime::UNIX_EPOCH)?; //Cicero checks lastmodified of CI executable to invalidate the cache when the build script changes.
        }
    }

    match Cli::parse() {
        Cli::Check(cli) => cli.run(),
        Cli::Coverage(cli) => cli.run(),
        Cli::Distribution(cli) => {
            use packages::carl::*;
            CarlCli { task: TaskCli::Distribution(cli) }.run()
        }
        Cli::Doc(cli) => cli.run(),
        Cli::Licenses(cli) => cli.run(PackageSelection::Applications),
        Cli::Test(cli) => cli.run(),
        Cli::IntegrationTest(cli) => cli.run(),

        Cli::Carl(cli) => cli.run(),
        Cli::Cleo(cli) => cli.run(),
        Cli::Edgar(cli) => cli.run(),
        Cli::Lea(cli) => cli.run(),
        Cli::Theo(cli) => cli.run(),
    }
}
