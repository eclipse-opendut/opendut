use clap::{Parser, Subcommand};
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
    /// Build a release distribution
    Distribution {
        #[arg(short, long)]
        package: Option<Package>,
        #[arg(short, long)]
        target: Arch,
    },
    /// Generate a license representation in JSON
    GenerateLicenses {
        #[arg()]
        package: Package,
    },
}

fn main() -> anyhow::Result<()> {
    init_tracing()?;

    std::env::set_current_dir(constants::workspace_dir())?;

    let cli = Cli::parse();
    match cli.task {
        Task::GenerateLicenses { package } => {
            tasks::licenses::generate_licenses(&package)?;
        }
        Task::Distribution { package, target } => {
            match package {
                Some(Package::OpendutCarl) => crate::packages::opendut_carl::distribution::carl(&target)?,
                Some(Package::OpendutEdgar) => crate::packages::opendut_edgar::distribution::edgar(&target)?,
                Some(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                None => {
                    //build distribution of everything
                    crate::packages::opendut_carl::distribution::carl(&target)?;
                    crate::packages::opendut_edgar::distribution::edgar(&target)?;
                },
            }
        }
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


use clap::builder::PossibleValue;
use strum::IntoEnumIterator;

impl clap::ValueEnum for Arch {
    fn value_variants<'a>() -> &'a [Arch] {
        Box::leak(Self::iter().collect::<Vec<Arch>>().into())
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            Arch::X86_64 => PossibleValue::new("x86_64-unknown-linux-gnu"),
            Arch::Armhf => PossibleValue::new("armv7-unknown-linux-gnueabihf"),
            Arch::Arm64 => PossibleValue::new("aarch64-unknown-linux-gnu"),
        })
    }
}
