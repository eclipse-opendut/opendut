use anyhow::anyhow;
use std::{io, env};
use std::path::{Path, PathBuf};


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO Error: {source}")]
    Io { #[from] source: io::Error },
    #[error("UTF8 Error: {source}")]
    Utf8 { #[from] source: std::str::Utf8Error },
    #[error("An error occurred: {source}")]
    Other { #[from] source: anyhow::Error },
}
type Result<T> = std::result::Result<T, Error>;


/// Takes a path and ensures it is absolute.
/// * Absolute paths will remain untouched.
/// * Relative paths will be resolved relative to:
///   * the workspace root, during development.
///   * the executable, when in production.
/// ```
/// use std::path::PathBuf;
/// use opendut_util::project;
///
/// let absolute_path = PathBuf::from("/tmp/test");
/// assert_eq!(project::make_path_absolute(&absolute_path).unwrap(), absolute_path);
///
/// let relative_path = PathBuf::from("tmp/test");
/// let result = project::make_path_absolute(&relative_path).unwrap();
/// assert!(result.is_absolute());
/// assert!(result.ends_with(relative_path));
/// ```
pub fn make_path_absolute(path: impl Into<PathBuf>) -> Result<PathBuf> {
    let path = path.into();
    let path = if path.is_absolute() {
        path
    } else {
        relative_root()?.join(path)
    };
    Ok(path)
}
fn relative_root() -> Result<PathBuf> {
    let path = if let Ok(cargo_executable) = env::var("CARGO") {
        workspace_dir(&cargo_executable)?
    } else {
        env::current_exe()?
            .parent().ok_or(anyhow!("Parent directory of current executable not found."))?
            .to_path_buf()
    };
    Ok(path)
}

pub fn is_running_in_development() -> bool {
    env::var("CARGO").is_ok()
}


/// Retrieve the directory at the root of the Cargo workspace. Only works in a development environment.
/// ```
/// use opendut_util::project::workspace_dir;
///
/// let path = if let Ok(cargo_executable) = std::env::var("CARGO") {
///     workspace_dir(&cargo_executable).unwrap()
/// } else {
///     todo!("Provide respective path when bundled for production use.")
/// };
/// ```
pub fn workspace_dir(cargo_executable: &str) -> Result<PathBuf> {

    let output = std::process::Command::new(cargo_executable)
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()?
        .stdout;

    let root_cargo_toml = Path::new(std::str::from_utf8(&output)?.trim());
    let workspace_dir = root_cargo_toml.parent().expect("Root Cargo.toml should have a parent directory.");

    Ok(workspace_dir.to_path_buf())
}
