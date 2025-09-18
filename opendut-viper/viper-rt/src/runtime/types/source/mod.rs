pub mod error;

use crate::runtime::types::source::error::InvalidSourceLocationError;
use std::path::PathBuf;
use url::Url;
use crate::common::TestSuiteIdentifier;

#[derive(Clone, Debug)]
pub struct Source {
    pub identifier: TestSuiteIdentifier,
    pub location: SourceLocation,
}

#[derive(Clone, Debug)]
pub enum SourceLocation {
    Embedded(String),
    Url(Url),
}

impl Source {

    fn new(name: TestSuiteIdentifier, inner: SourceLocation) -> Self {
        Self { identifier: name, location: inner }
    }

    pub fn embedded(code: impl Into<String>) -> Self {
        let identifier = TestSuiteIdentifier::new_embedded();
        Self::new(identifier, SourceLocation::Embedded(code.into()))
    }

    pub fn try_from_url(identifier: TestSuiteIdentifier, url: Url) -> Result<Self, InvalidSourceLocationError> {
        Ok(Self::new(identifier, SourceLocation::Url(url)))
    }

    pub fn try_from_url_str(identifier: TestSuiteIdentifier, url: &str) -> Result<Self, InvalidSourceLocationError> {
        let url = Url::parse(url)
            .map_err(|error| InvalidSourceLocationError::new_invalid_url_error(url, error.to_string()))?;
        Self::try_from_url(identifier, url)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn try_from_path(identifier: TestSuiteIdentifier, path: &PathBuf) -> Result<Self, InvalidSourceLocationError> {
        let url = Url::from_file_path(path)
            .map_err(|_| InvalidSourceLocationError::new_non_absolute_path_error(Clone::clone(path)))?;
        Self::try_from_url(identifier, url)
    }
}
