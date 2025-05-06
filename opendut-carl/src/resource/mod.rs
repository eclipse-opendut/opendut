use std::io;
use std::path::PathBuf;

pub(crate) mod api;
pub mod manager;
pub(crate) mod storage;
pub(crate) mod subscription;
pub mod persistence;

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Failed to load configuration.")]
    ConfigLoad(#[from] opendut_util::settings::LoadError),
    #[error("Failed to open or create database at {file:?}")]
    DatabaseCreate { file: PathBuf, #[source] source: redb::DatabaseError },
    #[error("Failed to create parent directory {dir:?} of database file")]
    DatabaseDirCreate { dir: PathBuf, #[source] source: io::Error },
    #[error("Failed to create database in-memory")]
    DatabaseInMemoryCreate(#[source] redb::DatabaseError),
}
