use std::io;
use std::path::PathBuf;

pub(crate) mod api;
pub mod manager;
pub(crate) mod storage;
pub(crate) mod subscription;
pub mod persistence;

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("Failed to open or create database at {file:?}")]
    DatabaseCreate { file: PathBuf, #[source] source: redb::DatabaseError },
    #[error("Failed to create parent directory {dir:?} of database file")]
    DatabaseDirCreate { dir: PathBuf, #[source] source: io::Error },


    #[cfg(feature="postgres")]
    #[error("Connection error from Diesel")]
    Diesel { url: url::Url, #[source] source: diesel::ConnectionError },

    #[cfg(feature="postgres")]
    #[error("Error while applying migrations")]
    Migration { #[source] source: Box<dyn std::error::Error + Send + Sync> },
}
