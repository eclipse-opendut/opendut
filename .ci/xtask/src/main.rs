use clap::{Parser, Subcommand};

pub(crate) use core::constants;
pub(crate) use core::metadata;
pub(crate) use core::types::{self, Arch, Package};
pub(crate) use core::util;

use crate::types::parsing::arch::ArchSelection;
use crate::types::parsing::package::PackageSelection;

mod core;
pub mod packages;
mod tasks;

shadow_rs::shadow!(build);


#[derive(Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    task: Task,
}
#[derive(Subcommand)]
enum Task {
    /// Perform a release build, without bundling a distribution.
    Build {
        #[arg(long, default_value_t)]
        target: ArchSelection,
    },
    /// Build and bundle a release distribution
    #[command(alias="dist")]
    Distribution {
        #[arg(long, default_value_t)]
        package: PackageSelection,
        #[arg(long, default_value_t)]
        target: ArchSelection,
    },
    /// Check or export licenses
    Licenses {
        #[command(subcommand)]
        task: tasks::licenses::LicensesTask,
    },

    #[command(alias="opendut-carl")]
    Carl {
        #[command(subcommand)]
        task: packages::carl::CarlTask,
    },
    #[command(alias="opendut-edgar")]
    Edgar {
        #[command(subcommand)]
        task: packages::edgar::EdgarTask,
    },
    #[command(alias="opendut-lea")]
    Lea {
        #[command(subcommand)]
        task: packages::lea::LeaTask,
    },
}

fn main() -> anyhow::Result<()> {
    util::init_tracing()?;

    std::env::set_current_dir(constants::workspace_dir())?;

    let cli = Cli::parse();
    match cli.task {
        Task::Build { target } => {
            for target in target.iter() {
                packages::carl::build::build_release(&target)?;
                packages::edgar::build::build_release(&target)?;
                packages::lea::build::build_release()?;
                //TODO build cleo
            }
        }
        Task::Distribution { package, target } => {
            for target in target.iter() {
                match package {
                    PackageSelection::Single(Package::Carl) => packages::carl::distribution::carl(&target)?,
                    PackageSelection::Single(Package::Edgar) => packages::edgar::distribution::edgar(&target)?,
                    PackageSelection::Single(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                    PackageSelection::All => {
                        //build distribution of everything
                        packages::carl::distribution::carl(&target)?;
                        packages::edgar::distribution::edgar(&target)?;
                        //TODO distribution of cleo
                    }
                }
            }
        }
        Task::Licenses { task } => tasks::licenses::LicensesTask::handle_task(task)?,

        Task::Carl { task } => packages::carl::CarlTask::handle_task(task)?,
        Task::Edgar { task } => packages::edgar::EdgarTask::handle_task(task)?,
        Task::Lea { task } => packages::lea::LeaTask::handle_task(task)?,
    };
    Ok(())
}
