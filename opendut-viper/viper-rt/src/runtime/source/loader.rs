use crate::source::Source;
use std::fmt::{Display, Formatter};

pub type SourceLoaderResult = Result<String, SourceLoaderError>;


#[async_trait::async_trait]
pub trait SourceLoader {
    fn identifier(&self) -> &str;
    fn supports(&self, source: &Source) -> bool;
    async fn load(&self, source: &Source) -> SourceLoaderResult;
}

#[derive(Clone, Debug)]
pub struct SourceLoaderError {
    message: String,
}

impl SourceLoaderError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl Display for SourceLoaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl core::error::Error for SourceLoaderError {}
