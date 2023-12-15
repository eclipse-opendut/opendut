use std::path::PathBuf;

use crate::{Arch, Package};
use crate::core::types::parsing::arch::ArchSelection;

const PACKAGE: &Package = &Package::Cleo;


#[derive(Debug, clap::Subcommand)]
pub enum CleoTask {
    Build {
        #[arg(long, default_value_t)]
        target: ArchSelection,
    },
}
impl CleoTask {
    #[tracing::instrument]
    pub fn handle_task(self) -> anyhow::Result<()> {
        match self {
            CleoTask::Build { target } => {
                for target in target.iter() {
                    build::build_release(&target)?;
                }
            }
        };
        Ok(())
    }
}


pub mod build {
    use super::*;

    pub fn build_release(target: &Arch) -> anyhow::Result<()> {
        crate::tasks::build::build_release(PACKAGE, target)
    }
    pub fn out_dir(target: &Arch) -> PathBuf {
        crate::tasks::build::out_dir(PACKAGE, target)
    }
}
