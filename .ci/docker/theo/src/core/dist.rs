use std::fs;
use std::io::ErrorKind;
use std::ops::Index;
use std::path::PathBuf;
use std::process::Command;
use anyhow::anyhow;

use crate::core::project::ProjectRootDir;
use crate::core::TARGET_TRIPLE;

fn make_distribution_with_cargo() -> crate::Result {
    println!("Creating distribution with cargo: 'cargo ci distribution'");
    let cargo_command = Command::new("cargo")
        .arg("ci")
        .arg("distribution")
        .status();
    match cargo_command {
        Ok(_exit_status) => {
            Ok(())
        }
        Err(error) => {
            match error.kind() {
                ErrorKind::NotFound => {
                    Err(anyhow!("Could not find 'cargo': {}", error))
                }
                _ => {
                    Err(anyhow!("Failed to create distribution with cargo: {}", error))
                }
            }
        }
    }
}

fn enumerate_distribution_tar_files(dist_path: PathBuf) -> Vec<String> {
    let paths = fs::read_dir(dist_path).unwrap();
    paths.into_iter()
        .map(|path| { path.unwrap().path() })
        .filter(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            file_name.contains("opendut") && file_name.contains(".tar.gz")
        })
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .collect::<Vec<_>>()
}

fn assert_exactly_one_distribution_of_each_component(expected_dist_files: &[&str], files: &[String]) -> crate::Result {
    for expected in expected_dist_files.iter().copied() {
        let filtered_existing_files = files.iter().filter(|&file| file.contains(expected)).cloned()
            .collect::<Vec<_>>();
        if filtered_existing_files.len() != 1 {
            return Err(anyhow!("There should be exactly one dist of '{}'. Found: {:?}", expected, filtered_existing_files));
        }
    }
    Ok(())
}

fn check_if_distribution_tar_exists_of_each_component(expected_dist_files: &[&str], files: Vec<String>) -> bool {
    let stripped_version_of_files = files.iter().cloned()
        .map(|file| {
            let pos = file.find(TARGET_TRIPLE).map(|i| i + 12).unwrap();
            file.index(..pos).to_owned()
        })
        .collect::<Vec<_>>();

    let count_existing_dist_files = expected_dist_files.iter().cloned().map(|expected| {
        stripped_version_of_files.contains(&expected.to_owned())
    });
    count_existing_dist_files.len() != expected_dist_files.len()
}


pub(crate) fn make_distribution_if_not_present() -> crate::Result {
    let dist_directory_path = PathBuf::project_path_buf()
        .join(format!("target/ci/distribution/{}", TARGET_TRIPLE));
    let expected_dist_files = vec!(
        //"opendut-cleo-linux-x86_64",
        "opendut-edgar-x86_64-unknown-linux-gnu",
        "opendut-carl-x86_64-unknown-linux-gnu",
    );

    if !dist_directory_path.exists() {
        println!("Distribution directory does not exist. Building distribution.");
        make_distribution_with_cargo()?;
    }

    let present_dist_files = enumerate_distribution_tar_files(dist_directory_path);
    assert_exactly_one_distribution_of_each_component(&expected_dist_files, &present_dist_files)?;

    if check_if_distribution_tar_exists_of_each_component(&expected_dist_files, present_dist_files) {
        make_distribution_with_cargo()?;
    }
    Ok(())
}
