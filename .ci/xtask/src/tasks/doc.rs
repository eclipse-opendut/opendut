use std::path::PathBuf;
use std::process::Command;

use crate::core::dependency::Crate;
use crate::util;
use crate::util::RunRequiringSuccess;

/// Access or build the documentation
#[derive(Debug, clap::Parser)]
pub struct DocCli {
    #[command(subcommand)]
    kind: DocKindCli,
}
#[derive(Debug, clap::Subcommand)]
enum DocKindCli {
    /// Long-form manual for openDuT
    Book {
        #[command(subcommand)]
        task: BookCli,
    },
}
#[derive(Debug, clap::Subcommand)]
enum BookCli {
    /// Serve the book for viewing in a browser.
    Open,
}

impl DocCli {
    pub fn default_handling(&self) -> anyhow::Result<()> {
        match &self.kind {
            DocKindCli::Book { task } => match task {
                BookCli::Open => book::serve()?,
            }
        };
        Ok(())
    }
}

pub mod book {
    use super::*;

    #[tracing::instrument]
    pub fn serve() -> anyhow::Result<()> {
        util::install_crate(Crate::Mdbook)?;
        util::install_crate(Crate::MdbookMermaid)?;

        Command::new("mdbook")
            .arg("serve")
            .arg("--open")
            .current_dir(doc_dir())
            .run_requiring_success();
        Ok(())
    }

    fn doc_dir() -> PathBuf {
        crate::constants::workspace_dir().join("doc")
    }
}
