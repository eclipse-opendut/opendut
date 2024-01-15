#[derive(thiserror::Error, Clone, Debug)]
pub(crate) enum TheoError {
    #[error("ConsumeOutputError Error: {0}")]
    ConsumeOutputError(String)
}

pub(crate) const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
pub(crate) const OPENDUT_REPO_ROOT: &str = "OPENDUT_REPO_ROOT";
pub(crate) const OPENDUT_THEO_DISABLE_ENV_CHECKS: &str = "OPENDUT_THEO_DISABLE_ENV_CHECKS";

pub(crate) mod project;
pub(crate) mod docker;
pub(crate) mod metadata;
pub(crate) mod util;
pub(crate) mod dist;

pub(crate) mod network;