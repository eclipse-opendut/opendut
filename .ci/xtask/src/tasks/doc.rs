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
    /// Create a distribution of the book.
    Build,
    /// Serve the book for viewing in a browser.
    Open,
}

impl DocCli {
    pub fn default_handling(&self) -> crate::Result {
        match &self.kind {
            DocKindCli::Book { task } => match task {
                BookCli::Build => book::build()?,
                BookCli::Open => book::open()?,
            }
        };
        Ok(())
    }
}

pub mod book {
    use super::*;

    #[tracing::instrument]
    pub fn open() -> crate::Result {
        util::install_crate(Crate::Mdbook)?;
        util::install_crate(Crate::MdbookPlantuml)?;

        Command::new("mdbook")
            .arg("serve")
            .arg("--open")
            .arg("--port=4000")
            .arg("--dest-dir").arg(out_dir())
            .current_dir(doc_dir())
            .run_requiring_success()?;
        Ok(())
    }

    #[tracing::instrument]
    pub fn build() -> crate::Result {
        util::install_crate(Crate::Mdbook)?;
        util::install_crate(Crate::MdbookPlantuml)?;

        Command::new("mdbook")
            .arg("build")
            .arg("--dest-dir").arg(out_dir())
            .current_dir(doc_dir())
            .run_requiring_success()?;
        Ok(())
    }

    fn doc_dir() -> PathBuf {
        crate::constants::workspace_dir().join("doc")
    }

    fn out_dir() -> PathBuf {
        crate::constants::target_dir().join("book")
    }
}
