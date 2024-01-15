#[derive(thiserror::Error, Clone, Debug)]
pub(crate) enum TheoError {
    #[error("ConsumeOutputError Error: {0}")]
    ConsumeOutputError(String)
}

pub(crate) const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
pub(crate) const OPENDUT_REPO_ROOT: &str = "OPENDUT_REPO_ROOT";

pub(crate) mod project;
pub(crate) mod docker;
pub(crate) mod metadata;
pub(crate) mod util;
pub(crate) mod dist;

pub(crate) mod network;