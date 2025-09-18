use std::path::PathBuf;

#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum InvalidSourceError {
    EmptySource,
    MissingViperVersion,
    IllegalViperVersionString(String),
    UnknownViperVersion(String),
}

impl InvalidSourceError {

    pub(crate) fn new_empty_source_error() -> Self {
        Self::EmptySource
    }

    pub(crate) fn new_missing_viper_version_error() -> Self {
        Self::MissingViperVersion
    }

    pub(crate) fn new_illegal_viper_version_string_error(value: impl Into<String>) -> Self {
        Self::IllegalViperVersionString(value.into())
    }

    pub(crate) fn new_unknown_viper_version_error(value: impl Into<String>) -> Self {
        Self::UnknownViperVersion(value.into())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum InvalidSourceLocationError {
    InvalidUrl { url: String, message: String },
    NonAbsolutePath { path: PathBuf },
}

impl InvalidSourceLocationError {

    pub(crate) fn new_invalid_url_error(
        url: impl Into<String>,
        message: impl Into<String>
    ) -> Self {
        Self::InvalidUrl {
            url: url.into(),
            message: message.into(),
        }
    }

    pub(crate) fn new_non_absolute_path_error(
        path: PathBuf,
    ) -> Self {
        Self::NonAbsolutePath {
            path
        }
    }
}
