use crate::runtime::source::loader::SourceLoaderResult;
use crate::runtime::source::{SourceLoader, SourceLoaderError};
use crate::runtime::types::source::SourceLocation;
use crate::source::Source;

/// A simple implementation of a [SourceLoader] that loads a [Source] from the local file system.
/// 
/// # Example
/// ```
/// use std::path::{absolute, PathBuf};
///
/// use viper_rt::events::emitter;
/// use viper_rt::source::Source;
/// use viper_rt::source::loaders::SimpleFileSourceLoader;
/// use viper_rt::ViperRuntime;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let runtime = ViperRuntime::builder()
///         .with_source_loader(SimpleFileSourceLoader)
///         .build()?;
///
///     let path = absolute(PathBuf::from("tests/minimal.py"))?;
///     let source = Source::try_from_path("minimal".try_into()?, &path)?;
///
///     runtime.compile(&source, &mut emitter::drain()).await?;
///
///     Ok(())
/// }
/// ```
/// <sup><b>Note:</b> This example uses [tokio](https://tokio.rs/), but viper is not bound to a specific async-runtime.</sup>
/// 
pub struct SimpleFileSourceLoader;

#[async_trait::async_trait]
impl SourceLoader for SimpleFileSourceLoader {

    fn identifier(&self) -> &str {
        "SimpleFileSourceLoader"
    }

    fn supports(&self, source: &Source) -> bool {
        if let SourceLocation::Url(url) = &source.location {
            url.scheme() == "file"
        }
        else {
            false
        }
    }

    async fn load(&self, source: &Source) -> SourceLoaderResult {

        let SourceLocation::Url(url) = &source.location else {
            return Err(SourceLoaderError::new("Cannot load embedded sources!"));
        };

        if url.scheme() != "file" {
            return Err(SourceLoaderError::new("Cannot load sources with non-file scheme!"));
        }

        let path = url.to_file_path()
            .map_err(|_| SourceLoaderError::new("Could not convert URL to file path!"))?;

        let content = std::fs::read_to_string(&path)
            .map_err(|error| SourceLoaderError::new(error.to_string()))?;

        Ok(content)
    }
}
