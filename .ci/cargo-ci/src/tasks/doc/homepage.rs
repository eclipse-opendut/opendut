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
        repo_path!("homepage/"),
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
