use crate::runtime::ctx::Context;
use crate::runtime::options::{ViperBuilder, ViperOptions};
use crate::source::loaders::EmbeddedSourceLoader;

#[cfg(feature = "compile")]
pub mod compile;

#[cfg(feature = "events")]
pub mod emitter;

#[cfg(feature = "run")]
pub mod run;

pub mod options;
pub mod source;
pub mod types;

mod ctx;
mod timeout;
mod error;

pub struct ViperRuntime {
    #[allow(dead_code)]
    context: Context,
}

impl Default for ViperRuntime {
    /// Creates a `ViperRuntime` with default [`ViperOptions`].
    /// 
    /// **Note:** The runtime contains an [`EmbeddedSourceLoader`] to load embedded sources.
    #[allow(clippy::needless_update)]
    fn default() -> Self {
        let options = ViperOptions {
            source_loaders: vec![Box::new(EmbeddedSourceLoader)],
            ..Default::default()
        };
        ViperRuntime::new(options)
            .expect("Default runtime should be valid")
    }
}

impl ViperRuntime {

    pub fn new(_options: ViperOptions) -> Result<Self, RuntimeInstantiationError> {
        Ok(Self {
            context: Context {
                #[cfg(feature = "compile")]
                source_loaders: _options.source_loaders,
                #[cfg(feature = "containers")]
                container_runtime: _options.container_runtime,
            },
        })
    }

    pub fn builder() -> ViperBuilder {
        ViperBuilder::default()
    }

    #[cfg(feature = "compile")]
    pub async fn compile_tree<Emitter>(
        &self,
        sources: Vec<(
            types::source::Source,
            Emitter,
        )>,
        identifier_filter: &crate::compile::IdentifierFilter,
    ) -> Result<
        Vec<types::compile::error::CompileResult<types::compile::compilation::Compilation>>,
        crate::runtime::types::compile::filter::FilterError
    >
    where Emitter: emitter::EventEmitter<types::compile::event::CompileEvent> + Send + 'static,
    {
        compile::compile_tree(sources, &self.context, identifier_filter).await
    }

    #[cfg(feature = "compile")]
    pub async fn compile(
        &self,
        source: &types::source::Source,
        emitter: &mut dyn emitter::EventEmitter<types::compile::event::CompileEvent>,
        identifier_filter: &crate::compile::IdentifierFilter,
    ) -> types::compile::error::CompileResult<types::compile::compilation::Compilation> {
        compile::compile(source, &self.context, emitter, identifier_filter).await
    }

    #[cfg(feature = "run")]
    pub async fn run(
        &self,
        suite: types::compile::suite::TestSuite,
        bindings: types::run::parameters::ParameterBindings<types::run::parameters::Complete>,
        emitter: &mut dyn emitter::EventEmitter<types::run::event::RunEvent>
    ) -> types::run::error::RunResult<types::run::report::TestSuiteReport> {
        run::run(suite, bindings, &self.context, emitter).await
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct RuntimeInstantiationError {
    pub message: String,
}
