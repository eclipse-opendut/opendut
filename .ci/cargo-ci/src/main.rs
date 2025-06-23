use cicero::distribution::filter::DistributionFilter;
use clap::Parser;

pub use fs_err as fs;
use cicero::path::repo_path;
use tracing::level_filters::LevelFilter;
pub(crate) use core::constants;
pub(crate) use core::metadata;
pub(crate) use core::types::{self, Arch, Package, Result};
pub(crate) use core::util;
use crate::core::types::parsing::package::PackageSelection;
use crate::tasks::distribution::bundle;

mod core;
pub mod packages;
mod tasks;

shadow_rs::shadow!(build);


#[derive(clap::Parser)]
struct RootCli {
    #[command(subcommand)]
    task: TaskCli,
}
#[derive(clap::Subcommand)]
enum TaskCli {
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

fn main() -> crate::Result {
    cicero::init::tracing()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    std::env::set_current_dir(repo_path!())?;

    let cli = RootCli::parse();
    match cli.task {
        TaskCli::Check(implementation) => implementation.default_handling()?,
        TaskCli::Coverage(implementation) => implementation.default_handling()?,
        TaskCli::Distribution(tasks::distribution::DistributionCli { target }) => {
            for target in target.iter() {
                packages::carl::distribution::carl_distribution(target)?;
                packages::edgar::distribution::edgar_distribution(target)?;

                let out_file = bundle::out_file(Package::Cleo, target);
                packages::cleo::distribution::cleo_distribution(target, &out_file, DistributionFilter::Disabled)?;
            }
        }
        TaskCli::Doc(implementation) => implementation.default_handling()?,
        TaskCli::Licenses(implementation) => implementation.default_handling(PackageSelection::Applications)?,
        TaskCli::Test(implementation) => implementation.default_handling()?,
        TaskCli::IntegrationTest(implementation) => implementation.default_handling()?,
        TaskCli::Venv(implementation) => implementation.run()?,

        TaskCli::Carl(implementation) => implementation.default_handling()?,
        TaskCli::Cleo(implementation) => implementation.default_handling()?,
        TaskCli::Edgar(implementation) => implementation.default_handling()?,
        TaskCli::Lea(implementation) => implementation.default_handling()?,
        TaskCli::Theo(implementation) => implementation.default_handling()?,
    };
    Ok(())
}
