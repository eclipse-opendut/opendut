use crate::fs;
use std::path::PathBuf;

use tracing::info;

use crate::util::RunRequiringSuccess;

mod book;
mod homepage;
mod latex;


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
    /// Build and pack homepage for openDuT
    Homepage {
        #[command(subcommand)]
        task: HomepageCli,
    },
}
#[derive(Debug, clap::Subcommand)]
enum BookCli {
    /// Create a distribution of the book.
    Build,
    /// Serve the book for viewing in a browser.
    Open,
    /// Create PNG images from LaTeX files.
    Images,
}

#[derive(Debug, clap::Subcommand)]
enum HomepageCli {
    /// Build the homepage
    Build,
}

impl DocCli {
    pub fn default_handling(&self) -> crate::Result {
        match &self.kind {
            DocKindCli::Book { task } => match task {
                BookCli::Build => {
                    book::build()?;
                    info!("Placed distribution into: {}", book::out_dir().display());
                },
                BookCli::Open => book::open()?,
                BookCli::Images => {
                    latex::create_images()?;
                    info!("Created images from LaTeX files.");
                }
            },
            DocKindCli::Homepage { task } => match task {
                HomepageCli::Build => {
                    homepage::build()?;
                    info!("Placed distribution into: {}", homepage::out_dir().display());
                },
            },
        };
        Ok(())
    }
}
