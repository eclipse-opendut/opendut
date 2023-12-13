use clap::{Parser, Subcommand};
use strum::IntoEnumIterator;
use tracing_subscriber::fmt::format::FmtSpan;

mod constants;
mod metadata;
pub mod packages;
mod tasks;
mod types;
pub use types::{Arch, Package};
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
        #[arg(long)]
        package: Option<Package>,
        #[arg(long)]
        target: Option<Arch>,
    },
    /// Build a release distribution
    Distribution {
        #[arg(long)]
        package: Option<Package>,
        #[arg(long)]
        target: Option<Arch>,
    },
    /// Generate a license representation in JSON
    GenerateLicenses {
        #[arg(long)]
        package: Option<Package>,
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
                Some(Package::Carl) => packages::carl::build::build_release(&target)?,
                Some(Package::Edgar) => packages::edgar::build::build_release(&target)?,
                Some(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                None => {
                    //build distribution of everything
                    packages::carl::build::build_release(&target)?;
                    packages::edgar::build::build_release(&target)?;
                }
            }
        }
        Task::GenerateLicenses { package } => {
            match package {
                Some(package) => tasks::licenses::generate_licenses(&package)?,
                None => {
                    for package in Package::iter() {
                        tasks::licenses::generate_licenses(&package)?
                    }
                }
            }
        }
        Task::Distribution { package, target } => {
            let target = Arch::get_or_default(target);
            match package {
                Some(Package::Carl) => packages::carl::distribution::carl(&target)?,
                Some(Package::Edgar) => packages::edgar::distribution::edgar(&target)?,
                Some(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                None => {
                    //build distribution of everything
                    packages::carl::distribution::carl(&target)?;
                    packages::edgar::distribution::edgar(&target)?;
                }
            }
        }
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
