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
        #[arg()]
        package: Package,
    },
    /// Start a development server for LEA which watches for file changes.
    LeaWatch,
}

fn main() -> anyhow::Result<()> {
    init_tracing()?;

    std::env::set_current_dir(constants::workspace_dir())?;

    let cli = Cli::parse();
    match cli.task {
        Task::Build { package, target } => {
            let target = target_or_default(target);
            match package {
                Some(Package::Carl) => crate::packages::carl::build_release(&target)?,
                Some(Package::Edgar) => crate::packages::edgar::build_release(&target)?,
                Some(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                None => {
                    //build distribution of everything
                    crate::packages::carl::build_release(&target)?;
                    crate::packages::edgar::build_release(&target)?;
                }
            }
        }
        Task::GenerateLicenses { package } => {
            tasks::licenses::generate_licenses(&package)?;
        }
        Task::Distribution { package, target } => {
            let target = target_or_default(target);
            match package {
                Some(Package::Carl) => crate::packages::carl::distribution::carl(&target)?,
                Some(Package::Edgar) => crate::packages::edgar::distribution::edgar(&target)?,
                Some(package) => unimplemented!("Building a distribution for {package} is not currently implemented."),
                None => {
                    //build distribution of everything
                    crate::packages::carl::distribution::carl(&target)?;
                    crate::packages::edgar::distribution::edgar(&target)?;
                }
            }
        }
        Task::LeaWatch => crate::packages::lea::lea_watch()?,
    };
    Ok(())
}

fn target_or_default(target: Option<Arch>) -> Arch {
    use clap::ValueEnum;

    target.unwrap_or_else(|| {
        let arch_triple = crate::build::BUILD_TARGET;
        log::info!("No target specified. Using default target of machine: {arch_triple}");
        let ignore_case = true;
        Arch::from_str(arch_triple, ignore_case).unwrap()
    })
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


mod parsing {
    use super::*;

    use clap::builder::PossibleValue;
    use strum::IntoEnumIterator;

    impl clap::ValueEnum for Arch {
        fn value_variants<'a>() -> &'a [Arch] {
            Box::leak(Self::iter().collect::<Vec<Arch>>().into())
        }
        fn to_possible_value(&self) -> Option<PossibleValue> {
            Some(PossibleValue::new(self.triple()))
        }
    }

    impl clap::ValueEnum for Package {
        fn value_variants<'a>() -> &'a [Package] {
            Box::leak(Self::iter().collect::<Vec<Package>>().into())
        }
        fn to_possible_value(&self) -> Option<PossibleValue> {
            Some(PossibleValue::new(self.ident()))
        }
    }
}
