use clap::Parser;

pub(crate) use core::constants;
pub(crate) use core::metadata;
pub(crate) use core::types::{self, Arch, Package};
pub(crate) use core::util;

mod core;
pub mod packages;
mod tasks;

shadow_rs::shadow!(build);


#[derive(clap::Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    task: Task,
}
#[derive(clap::Subcommand)]
enum Task {
    Build(tasks::build::Build),
    Distribution(tasks::distribution::Distribution),
    Licenses(tasks::licenses::LicensesCli),

    Carl(packages::carl::CarlCli),
    Cleo(packages::cleo::CleoCli),
    Edgar(packages::edgar::EdgarCli),
    Lea(packages::lea::LeaCli),
}

fn main() -> anyhow::Result<()> {
    util::init_tracing()?;

    std::env::set_current_dir(constants::workspace_dir())?;

    let cli = Cli::parse();
    match cli.task {
        Task::Build(tasks::build::Build { target }) => {
            for target in target.iter() {
                packages::carl::build::build_release(&target)?;
                packages::cleo::build::build_release(&target)?;
                packages::edgar::build::build_release(&target)?;
                packages::lea::build::build_release()?;
            }
        }
        Task::Distribution(tasks::distribution::Distribution { target }) => {
            for target in target.iter() {
                packages::carl::distribution::carl_distribution(&target)?;
                packages::edgar::distribution::edgar_distribution(&target)?;
                packages::cleo::distribution::cleo_distribution(&target)?;
            }
        }
        Task::Licenses(implementation) => implementation.handle()?,

        Task::Carl(implementation) => implementation.handle()?,
        Task::Cleo(implementation) => implementation.handle()?,
        Task::Edgar(implementation) => implementation.handle()?,
        Task::Lea(implementation) => implementation.handle()?,
    };
    Ok(())
}
