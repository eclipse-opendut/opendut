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

pub fn workspace_dir() -> PathBuf {
    WORKSPACE_DIR.to_path_buf()
}

pub fn target_dir() -> PathBuf {
    workspace_dir().join("target")
}

pub fn ci_dir() -> PathBuf {
    target_dir().join("ci")
}
