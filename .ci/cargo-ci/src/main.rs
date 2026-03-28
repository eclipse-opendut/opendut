use clap::Parser;

pub use fs_err as fs;
use tracing_subscriber::EnvFilter;
pub(crate) use core::constants;
pub(crate) use core::metadata;
pub(crate) use core::types::Package;
pub(crate) use core::util;
use crate::core::types::parsing::package::PackageSelection;

mod core;
pub mod packages;
mod tasks;

shadow_rs::shadow!(build);


#[derive(clap::Parser)]
enum Cli {
    Check(tasks::check::CheckCli),
    Coverage(tasks::coverage::CoverageCli),
    Distribution(tasks::distribution::DistributionCli),
    Doc(tasks::doc::DocCli),
    Licenses(tasks::licenses::LicensesCli),
    Test(tasks::test::TestCli),
    IntegrationTest(tasks::integration_tests::IntegrationTestCli),
    Venv(cicero::commands::venv::VenvCli),

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

    match Cli::parse() {
        Cli::Check(cli) => cli.run(),
        Cli::Coverage(cli) => cli.run(),
        Cli::Distribution(tasks::distribution::DistributionCli { target, release_build }) => {
            packages::carl::distribution::carl_distribution(target, release_build)
        }
        Cli::Doc(cli) => cli.run(),
        Cli::Licenses(cli) => cli.run(PackageSelection::Applications),
        Cli::Test(cli) => cli.run(),
        Cli::IntegrationTest(cli) => cli.run(),
        Cli::Venv(cli) => cli.run(),

        Cli::Carl(cli) => cli.run(),
        Cli::Cleo(cli) => cli.run(),
        Cli::Edgar(cli) => cli.run(),
        Cli::Lea(cli) => cli.run(),
        Cli::Theo(cli) => cli.run(),
    }
}
