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

pub mod latex {
    use std::path::Path;
    use std::process::Command;
    use anyhow::{anyhow, Context};
    use cicero::path::repo_path;
    use walkdir::WalkDir;
    use super::*;

    fn enumerate_tex_files() -> Vec<PathBuf> {
        let repo_root = repo_path!();
        let exclude_dirs = ["target"];

        WalkDir::new(repo_root.clone())
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                let excluded_directory = exclude_dirs.iter().any(|ex| entry.path().starts_with(repo_root.join(ex)));
                let is_latex_file = entry.path().extension().map(|ext| ext == "tex").unwrap_or(false);

                !excluded_directory && is_latex_file
            })
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    fn create_pdf_file(latex_file: &PathBuf, working_dir: &Path) -> crate::Result {
        let pdf_status = Command::new("pdflatex")
            .arg("-interaction=nonstopmode")
            .arg("-halt-on-error")
            .arg(latex_file)
            .current_dir(working_dir)
            .status()
            .context("Failed to run pdflatex.")?;

        if !pdf_status.success() {
            return Err(anyhow!("Failed to compile LaTeX file: {}", latex_file.display()));
        }

        Ok(())
    }

    fn convert_pdf_to_png_image(pdf_file: &PathBuf, png_file: &PathBuf) -> crate::Result {
        // https://imagemagick.org/script/command-line-options.php#quality
        let convert_status = Command::new("convert")
            .arg("-density").arg("300")  // 300 dpi
            .arg(pdf_file)
            .arg("-quality").arg("90")   // compression level and quality level
            .arg(png_file)
            .status()
            .context("Failed to convert LaTeX file to image using the ImageMagick command 'convert'.")?;

        if !convert_status.success() {
            return Err(anyhow!("Failed to convert PDF to PNG for file: {}", pdf_file.display()));
        }

        Ok(())
    }

    fn cleanup_auxiliary_files(latex_file: &Path) -> crate::Result {
        let aux_extensions = ["aux", "log", "out", "pdf"];
        for ext in &aux_extensions {
            let aux_file = latex_file.with_extension(ext);
            if aux_file.exists() {
                fs::remove_file(aux_file)?;
            }
        }
        Ok(())
    }

    #[tracing::instrument]
    pub fn create_images() -> crate::Result {
        let latex_files = enumerate_tex_files();

        for file in latex_files {
            let pdf_file = file.with_extension("pdf");
            let png_file = file.with_extension("png");

            let working_dir = file.parent();
            if let Some(working_dir) = working_dir {
                info!("Processing LaTeX file: {}", file.display());
                create_pdf_file(&file, working_dir)?;
                convert_pdf_to_png_image(&pdf_file, &png_file)?;
                cleanup_auxiliary_files(&file)?;
                info!("Created image: {}", png_file.display());
            }

        }

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
