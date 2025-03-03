use crate::fs;
use std::path::PathBuf;

use tracing::info;

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

pub mod book {
    use cicero::path::repo_path;
    use crate::core::commands::MDBOOK;
    use super::*;

    #[tracing::instrument]
    pub fn open() -> crate::Result {
        MDBOOK.command()
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
        let out_dir = out_dir();

        MDBOOK.command()
            .arg("build")
            .arg("--dest-dir").arg(&out_dir)
            .current_dir(doc_dir())
            .run_requiring_success()?;

        Ok(())
    }

    fn doc_dir() -> PathBuf {
        repo_path!("doc/")
    }

    pub fn out_dir() -> PathBuf {
        crate::constants::target_dir().join("book")
    }
}

pub mod homepage {
    use cicero::path::repo_path;
    use super::*;

    #[tracing::instrument]
    pub fn build() -> crate::Result {
        fs::create_dir_all(out_dir())?;

        book::build()?;
        fs_extra::dir::copy(
            book::out_dir(),
            out_dir().join("book"),
            &fs_extra::dir::CopyOptions::default()
                .overwrite(true)
                .content_only(true)
        )?;

        fs_extra::dir::copy(
            repo_path!("opendut-homepage/"),
            out_dir(),
            &fs_extra::dir::CopyOptions::default()
                .overwrite(true)
                .content_only(true)
        )?;

        fs_extra::dir::create(
            logos_out_dir(),
            true
        )?;

        for logo in RESOURCES_TO_INCLUDE {
            fs_extra::file::copy(
                repo_path!("resources/logos/").join(logo),
                logos_out_dir().join(logo),
                &fs_extra::file::CopyOptions::default()
                    .overwrite(true)
            )?;
        }

        Ok(())
    }

    pub fn out_dir() -> PathBuf { crate::constants::target_dir().join("homepage") }

    fn logos_out_dir() -> PathBuf { out_dir().join("resources/logos") }

    const RESOURCES_TO_INCLUDE: [&str; 2] = ["logo_light.png", "funded_by_the_european_union.svg"];
}
