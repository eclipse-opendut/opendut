mod loader;

pub use loader::{SourceLoader, SourceLoaderError, SourceLoaderResult};

pub mod embedded;

#[cfg(feature = "file-source")]
pub mod file;

#[cfg(feature = "git-source")]
pub mod git;

#[cfg(feature = "http-source")]
pub mod http;
