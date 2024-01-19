use std::path::{PathBuf, Path};

use lazy_static::lazy_static;

lazy_static! {
    static ref WORKSPACE_DIR: PathBuf = {
        let output = std::process::Command::new(env!("CARGO"))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .unwrap()
            .stdout;
        let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
        cargo_path.parent().unwrap().to_path_buf()
    };
}

/// The root of the Cargo Workspace. Should be the repository root.
pub fn workspace_dir() -> PathBuf {
    WORKSPACE_DIR.to_owned()
}

/// Where CI-related code is in the repository.
pub fn ci_dir() -> PathBuf { workspace_dir().join(".ci") }

/// The generic Cargo target directory.
pub fn cargo_target_dir() -> PathBuf {
    workspace_dir().join("target")
}

/// Sub-directory of the Cargo target directory, which we use for any of our own build artifacts.
pub fn target_dir() -> PathBuf {
    cargo_target_dir().join("ci")
}
