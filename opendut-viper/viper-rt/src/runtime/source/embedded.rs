use crate::runtime::source::loader::SourceLoaderResult;
use crate::runtime::source::{SourceLoader, SourceLoaderError};
use crate::runtime::types::source::SourceLocation;
use crate::source::{Source};

/// A [SourceLoader] that loads a [Source] from embedded code.
/// 
/// # Example
/// ```
/// use indoc::indoc;
///
/// use viper_rt::events::emitter;
/// use viper_rt::run::ParameterBindings;
/// use viper_rt::source::Source;
/// use viper_rt::ViperRuntime;
/// use viper_rt::source::loaders::EmbeddedSourceLoader;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let runtime = ViperRuntime::builder()
///         .with_source_loader(EmbeddedSourceLoader)
///         .build()?;
///
///     runtime.compile(&Source::embedded(
///         indoc!(r#"
///             ## VIPER_VERSION = 1.0
///             from viper import unittest
///             
///             class MyTestCase(unittest.TestCase):
///                 def test_awesomeness(self):
///                     print("Awesome!")
///         "#)
///     ), &mut emitter::drain()).await?;
///
///     Ok(())
/// }
/// ```
/// <sup><b>Note:</b> This example uses [tokio](https://tokio.rs/), but viper is not bound to a specific async-runtime.</sup>
/// 
pub struct EmbeddedSourceLoader;

#[async_trait::async_trait]
impl SourceLoader for EmbeddedSourceLoader {

    fn identifier(&self) -> &str {
        "EmbeddedSourceLoader"
    }

    fn supports(&self, source: &Source) -> bool {
        matches!(source.location, SourceLocation::Embedded(_))
    }

    async fn load(&self, source: &Source) -> SourceLoaderResult {
        let SourceLocation::Embedded(code) = &source.location else {
            return Err(SourceLoaderError::new("Cannot load source from Url!"));
        };
        Ok(Clone::clone(code))
    }
}
