use crate::runtime::RuntimeInstantiationError;
use crate::runtime::source::SourceLoader;
use crate::ViperRuntime;

#[derive(Default)]
pub struct ViperOptions {
    pub source_loaders: Vec<Box<dyn SourceLoader>>,
    #[cfg(feature = "containers")]
    pub container_runtime: Option<crate::containers::ContainerRuntime>,
}

#[derive(Default)]
pub struct ViperBuilder {
    options: ViperOptions,
}

impl ViperBuilder {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_source_loader<L>(mut self, loader: L) -> Self
    where
        L: SourceLoader + 'static,
    {
        self.options.source_loaders.push(Box::new(loader));
        self
    }

    #[cfg(feature = "containers")]
    pub fn with_container_runtime(mut self, runtime: crate::containers::ContainerRuntime) -> Self {
        self.options.container_runtime = Some(runtime);
        self
    }

    pub fn build(self) -> Result<ViperRuntime, RuntimeInstantiationError> {
        ViperRuntime::new(self.options)
    }
}
