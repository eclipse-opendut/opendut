use std::path::PathBuf;
use cicero::path::repo_path;

/// Sub-directory of the Cargo target directory, which we use for any of our own build artifacts.
pub fn target_dir() -> PathBuf {
    repo_path!().join("target/ci/")
}
