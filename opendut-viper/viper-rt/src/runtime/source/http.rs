use crate::runtime::source::loader::SourceLoaderResult;
use crate::runtime::source::{SourceLoader, SourceLoaderError};
use crate::source::{Source, SourceLocation};
use futures::TryFutureExt;
use reqwest::Response;

/// A simple implementation of a [SourceLoader] that loads a [Source] from an [HTTP](https://en.wikipedia.org/wiki/HTTP) server.
///
/// # Example
/// ```
/// # use httpmock::prelude::*;
/// # use indoc::indoc;
/// use viper_rt::events::emitter;
/// use viper_rt::source::loaders::HttpSourceLoader;
/// use viper_rt::source::Source;
/// use viper_rt::ViperRuntime;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// #  let server = MockServer::start();
/// #  let mock = server.mock(|when, then| {
/// #      when.method(GET)
/// #          .path("/testsuite.py");
/// #      then.status(200)
/// #          .body(
/// #              indoc!(r#"
/// #                  # VIPER_VERSION = 1.0
/// #                  from viper import unittest
/// #                  class SomeClass(unittest.TestCase): 
/// #                      def test_awesomeness(self):
/// #                          print("Awesome!")
/// #                  "#)
/// #              );
/// #      });
///
///   let runtime = ViperRuntime::builder()
///     .with_source_loader(HttpSourceLoader)
///     .build()?;
///
///   let url = String::from("http://example.com/testsuite.py");
/// # let url = server.url("/testsuite.py");
///   let source = Source::try_from_url_str("example".try_into()?, &url)?;
///
///   runtime.compile(&source, &mut emitter::drain()).await?;
///
/// #   mock.assert();
/// #
///     Ok(())
/// }
/// ```
/// <sup><b>Note:</b> This example uses [tokio](https://tokio.rs/), but viper is not bound to a specific async-runtime.</sup>
///
pub struct HttpSourceLoader;

impl HttpSourceLoader {

    pub fn new(_option: HttpSourceLoaderOptions) -> Self {
        Self
    }
}

impl Default for HttpSourceLoader {

    fn default() -> Self {
        HttpSourceLoader::new(HttpSourceLoaderOptions::default())   
    }
}

#[async_trait::async_trait]
impl SourceLoader for HttpSourceLoader {

    fn identifier(&self) -> &str {
        "HttpSourceLoader"
    }

    fn supports(&self, source: &Source) -> bool {
        let SourceLocation::Url(url) = &source.location else {
            return false
        };
        let scheme = url.scheme();
        scheme == "http" || scheme == "https"
    }

    async fn load(&self, source: &Source) -> SourceLoaderResult {

        let SourceLocation::Url(url) = &source.location else {
            return Err(SourceLoaderError::new("Invalid source location"))
        };

        let response: Response = reqwest::get(Clone::clone(url))
            .map_err(|error| SourceLoaderError::new(error.to_string()))
            .await?;

        if !response.status().is_success() {
            return Err(SourceLoaderError::new(response.status().to_string()))
        };

        let content = response
            .text()
            .map_err(|error| SourceLoaderError::new(error.to_string()))
            .await?;

        Ok(content)
    }
}

#[derive(Default)]
pub struct HttpSourceLoaderOptions {
}
