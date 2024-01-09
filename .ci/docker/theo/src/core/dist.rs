use std::fs;
use std::ops::Index;
use std::path::PathBuf;
use std::process::Command;

use crate::core::project::ProjectRootDir;
use crate::core::TARGET_TRIPLE;

fn make_distribution_with_cargo() {
    println!("Create distribution with cargo: 'cargo ci distribution'");
    let _dist_status = Command::new("cargo")
        .arg("ci")
        .arg("distribution")
        .status()
        .expect("Failed to update distribution");
}

fn enumerate_distribution_tar_files(dist_path: PathBuf) -> Vec<String> {
    let paths = fs::read_dir(dist_path).unwrap();
    let files = paths.into_iter()
        .map(|path| { path.unwrap().path() })
        .filter(|path| {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            file_name.contains("opendut") && file_name.contains(".tar.gz")
        })
        .map(|path| path.file_name().unwrap().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    files
}

fn assert_exactly_one_distribution_of_each_component(expected_dist_files: &Vec<&str>, files: &Vec<String>) {
    for expected in expected_dist_files.clone() {
        let filtered_existing_files = files.iter().cloned()
            .filter(|file| file.contains(expected))
            .collect::<Vec<_>>();
        assert_eq!(filtered_existing_files.len(), 1,
                   "There should be exactly one dist of '{}'. Found: {:?}", expected, filtered_existing_files);
    }
}

fn check_if_distribution_tar_exists_of_each_component(expected_dist_files: &Vec<&str>, files: Vec<String>) -> bool {
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


pub(crate) fn make_distribution_if_not_present() {
    let dist_directory_path = PathBuf::project_path_buf()
        .join(format!("target/ci/distribution/{}", TARGET_TRIPLE));
    let expected_dist_files = vec!(
        //"opendut-cleo-linux-x86_64",
        "opendut-edgar-x86_64-unknown-linux-gnu",
        "opendut-carl-x86_64-unknown-linux-gnu",
    );

    if !dist_directory_path.exists() {
        make_distribution_with_cargo();
    }

    let present_dist_files = enumerate_distribution_tar_files(dist_directory_path);
    assert_exactly_one_distribution_of_each_component(&expected_dist_files, &present_dist_files);

    if check_if_distribution_tar_exists_of_each_component(&expected_dist_files, present_dist_files) {
        make_distribution_with_cargo();
    }

}
