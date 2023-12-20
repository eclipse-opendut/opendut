use clap::Parser;

pub(crate) use core::constants;
pub(crate) use core::metadata;
pub(crate) use core::types::{self, Arch, Target, Package};
pub(crate) use core::util;
use crate::core::types::parsing::package::PackageSelection;

mod core;
pub mod packages;
mod tasks;

shadow_rs::shadow!(build);


#[derive(clap::Parser)]
#[command()]
struct RootCli {
    #[command(subcommand)]
    task: TaskCli,
}
#[derive(clap::Subcommand)]
enum TaskCli {
    Build(tasks::build::BuildCli),
    Distribution(tasks::distribution::DistributionCli),
    Licenses(tasks::licenses::LicensesCli),

    Carl(packages::carl::CarlCli),
    Cleo(packages::cleo::CleoCli),
    Edgar(packages::edgar::EdgarCli),
    Lea(packages::lea::LeaCli),
}

fn main() -> anyhow::Result<()> {
    util::init_tracing()?;

    std::env::set_current_dir(constants::workspace_dir())?;

    let cli = RootCli::parse();
    match cli.task {
        TaskCli::Build(tasks::build::BuildCli { target }) => {
            for target in target.iter() {
                packages::carl::build::build_release(&target)?;
                packages::cleo::build::build_release(&target)?;
                packages::edgar::build::build_release(&target)?;
                packages::lea::build::build_release()?;
            }
        }
        TaskCli::Distribution(tasks::distribution::DistributionCli { target }) => {
            for target in target.iter() {
                packages::carl::distribution::carl_distribution(&target)?;
                packages::edgar::distribution::edgar_distribution(&target)?;
                packages::cleo::distribution::cleo_distribution(&target)?;
            }
        }
        TaskCli::Licenses(implementation) => implementation.default_handling(PackageSelection::All)?,

        TaskCli::Carl(implementation) => implementation.default_handling()?,
        TaskCli::Cleo(implementation) => implementation.default_handling()?,
        TaskCli::Edgar(implementation) => implementation.default_handling()?,
        TaskCli::Lea(implementation) => implementation.default_handling()?,
    };
    Ok(())
}
