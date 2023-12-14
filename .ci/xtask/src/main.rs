use clap::{Parser, Subcommand};
use tracing_subscriber::fmt::format::FmtSpan;

pub use types::{Arch, Package};
use crate::types::package::PackageSelection;

mod constants;
mod metadata;
pub mod packages;
mod tasks;
mod types;
mod util;

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
        package: PackageSelection,
        #[arg(long)]
        target: Option<Arch>,
    },
    /// Build a release distribution
    Distribution {
        #[arg(long, default_value_t)]
        package: PackageSelection,
        #[arg(long)]
        target: Option<Arch>,
    },
    Licenses {
        #[command(subcommand)]
        task: tasks::licenses::LicensesTask,
    },
    Lea {
        #[command(subcommand)]
        task: packages::lea::LeaTask,
    },
    Edgar {
        #[command(subcommand)]
        task: packages::edgar::EdgarTask,
    }
}

fn main() -> anyhow::Result<()> {
    init_tracing()?;

    std::env::set_current_dir(constants::workspace_dir())?;

    let cli = Cli::parse();
    match cli.task {
        Task::Build { package, target } => {
            let target = Arch::get_or_default(target);
            match package {
                PackageSelection::Single(Package::Carl) => packages::carl::build::build_release(&target)?,
                PackageSelection::Single(Package::Edgar) => packages::edgar::build::build_release(&target)?,
                PackageSelection::Single(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                PackageSelection::All => {
                    //build distribution of everything
                    packages::carl::build::build_release(&target)?;
                    packages::edgar::build::build_release(&target)?;
                }
            }
        }
        Task::Distribution { package, target } => {
            let target = Arch::get_or_default(target);
            match package {
                PackageSelection::Single(Package::Carl) => packages::carl::distribution::carl(&target)?,
                PackageSelection::Single(Package::Edgar) => packages::edgar::distribution::edgar(&target)?,
                PackageSelection::Single(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                PackageSelection::All => {
                    //build distribution of everything
                    packages::carl::distribution::carl(&target)?;
                    packages::edgar::distribution::edgar(&target)?;
                }
            }
        }
        Task::Licenses { task } => tasks::licenses::LicensesTask::handle_task(task)?,
        Task::Lea { task } => packages::lea::LeaTask::handle_task(task)?,
        Task::Edgar { task } => packages::edgar::EdgarTask::handle_task(task)?,
    };
    Ok(())
}

fn init_tracing() -> anyhow::Result<()> {
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};

    let tracing_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()?
        .add_directive("opendut=trace".parse()?);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_span_events(FmtSpan::ENTER | FmtSpan::CLOSE)
        .with_env_filter(tracing_filter)
        .compact()
        .init();
    Ok(())
}
